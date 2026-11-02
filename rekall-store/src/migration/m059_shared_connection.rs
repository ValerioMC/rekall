//! `059`: a connection belongs to the workspace and is shared with companies through
//! `connection_company`, so it no longer carries a company. Its label becomes unique across the
//! workspace: a later connection that took an earlier one's label gets `-2`, `-3`... after it, in
//! the order the connections were created. A GitLab or GitHub connection may name the root path
//! its credential covers.

use std::collections::HashSet;

use sea_orm::{ConnectionTrait, DbErr, Statement};
use sea_orm_migration::prelude::SchemaManager;

use super::{changeset, exec};
use crate::connection::entity::external_connection::LABEL_MAX_CHARACTERS;

changeset!(SharedConnection, "059-shared-connection", |manager| {
    exec(manager, &[
        "CREATE TABLE connection_company (
            connection_id TEXT NOT NULL,
            company_id TEXT NOT NULL,
            CONSTRAINT pk_connection_company PRIMARY KEY (connection_id, company_id),
            CONSTRAINT fk_connection_company_connection FOREIGN KEY (connection_id) REFERENCES external_connection (id) ON DELETE CASCADE,
            CONSTRAINT fk_connection_company_company FOREIGN KEY (company_id) REFERENCES company (id) ON DELETE CASCADE)",
        "CREATE INDEX ix_connection_company_company ON connection_company (company_id)",
        "INSERT INTO connection_company (connection_id, company_id) SELECT id, company_id FROM external_connection",
    ])
    .await?;
    make_labels_unique(manager).await?;
    exec(manager, &[
        "CREATE TABLE external_connection_shared (
            id TEXT NOT NULL CONSTRAINT pk_external_connection PRIMARY KEY,
            label VARCHAR(120) NOT NULL,
            title VARCHAR(200) NOT NULL,
            kind VARCHAR(20) NOT NULL,
            base_url VARCHAR(1000),
            kube_context VARCHAR(253),
            proxy_url VARCHAR(1000),
            root_path VARCHAR(300),
            secret_ref VARCHAR(80) NOT NULL,
            secret_set_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            CONSTRAINT uq_external_connection_label UNIQUE (label),
            CONSTRAINT uq_external_connection_secret_ref UNIQUE (secret_ref))",
        "INSERT INTO external_connection_shared (id, label, title, kind, base_url, kube_context, proxy_url, secret_ref, secret_set_at, created_at, updated_at)
            SELECT id, label, title, kind, base_url, kube_context, proxy_url, secret_ref, secret_set_at, created_at, updated_at
            FROM external_connection",
        "DROP TABLE external_connection",
        "ALTER TABLE external_connection_shared RENAME TO external_connection",
    ])
    .await
});

async fn make_labels_unique(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let connection = manager.get_connection();
    let rows = connection
        .query_all_raw(Statement::from_string(connection.get_database_backend(), "SELECT id, label FROM external_connection ORDER BY created_at, id"))
        .await?;
    let mut labelled = Vec::with_capacity(rows.len());
    for row in &rows {
        labelled.push((row.try_get_by_index::<String>(0)?, row.try_get_by_index::<String>(1)?));
    }
    for (id, label) in renamed_labels(&labelled) {
        connection
            .execute_raw(Statement::from_sql_and_values(connection.get_database_backend(), "UPDATE external_connection SET label = ? WHERE id = ?", [label.into(), id.into()]))
            .await?;
    }
    Ok(())
}

/// The connections that have to change label, in the order given: the first holder of a label keeps it.
fn renamed_labels(labelled: &[(String, String)]) -> Vec<(String, String)> {
    let mut taken: HashSet<String> = HashSet::new();
    let mut duplicates: Vec<&(String, String)> = Vec::new();
    for entry in labelled {
        if !taken.insert(entry.1.clone()) {
            duplicates.push(entry);
        }
    }
    duplicates
        .into_iter()
        .map(|(id, label)| {
            let free = (2..).map(|suffix| suffixed(label, suffix)).find(|candidate| taken.insert(candidate.clone()));
            (id.clone(), free.expect("an unbounded range always offers a free label"))
        })
        .collect()
}

fn suffixed(label: &str, suffix: u32) -> String {
    let tail = format!("-{suffix}");
    let kept: String = label.chars().take(LABEL_MAX_CHARACTERS - tail.chars().count()).collect();
    format!("{kept}{tail}")
}
