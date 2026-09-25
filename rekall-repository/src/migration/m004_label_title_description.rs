//! `004-label-title-description.yaml`: the name a record was called by splits into a `label`,
//! which is what an anchor resolves and is normalised into a slug, and a free `title`.

use std::sync::LazyLock;

use regex::Regex;
use sea_orm::{ConnectionTrait, DbErr, Statement, Value};
use sea_orm_migration::SchemaManager;

use super::{changeset, exec, rebuild};

/// The changeset's own normalising expression,
/// `REGEXP_REPLACE(REGEXP_REPLACE(LOWER(label), '[^a-z0-9._-]+', '-'), '^[._-]+|[._-]+$', '')`,
/// which SQLite has no function for. It is not `Slug.of`: it does not fold a run of separators
/// into one, and that difference is kept.
pub fn legacy_label(name: &str) -> String {
    static OTHER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[^a-z0-9._-]+").unwrap());
    static EDGES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[._-]+|[._-]+$").unwrap());
    let lowered = name.to_lowercase();
    let replaced = OTHER.replace_all(&lowered, "-");
    EDGES.replace_all(&replaced, "").into_owned()
}

async fn normalise_labels(manager: &SchemaManager<'_>, table: &str) -> Result<(), DbErr> {
    let conn = manager.get_connection();
    let backend = conn.get_database_backend();
    let rows = conn
        .query_all_raw(Statement::from_string(backend, format!("SELECT id, label FROM {table}")))
        .await?;
    for row in rows {
        let id: String = row.try_get_by_index(0)?;
        let label: String = row.try_get_by_index(1)?;
        let normalised = legacy_label(&label);
        if normalised != label {
            conn.execute_raw(Statement::from_sql_and_values(
                backend,
                format!("UPDATE {table} SET label = ? WHERE id = ?"),
                [Value::from(normalised), Value::from(id)],
            ))
            .await?;
        }
    }
    Ok(())
}

changeset!(ProjectLabelAndTitle, "004-project-label-and-title", |manager| {
    exec(manager, &[
        "ALTER TABLE project ADD COLUMN title VARCHAR(200)",
        "UPDATE project SET title = name WHERE title IS NULL",
        "ALTER TABLE project RENAME COLUMN name TO label",
    ])
    .await?;
    normalise_labels(manager, "project").await?;
    rebuild(
        manager,
        "project",
        "id TEXT NOT NULL CONSTRAINT pk_project PRIMARY KEY,
         label VARCHAR(120) NOT NULL,
         status VARCHAR(20) NOT NULL,
         description VARCHAR(100000),
         created_at TEXT NOT NULL,
         updated_at TEXT NOT NULL,
         company_id TEXT NOT NULL,
         title VARCHAR(200) NOT NULL,
         CONSTRAINT fk_project_company FOREIGN KEY (company_id) REFERENCES company (id) ON DELETE CASCADE,
         CONSTRAINT uq_project_company_label UNIQUE (company_id, label)",
        "id, label, status, description, created_at, updated_at, company_id, title",
        &[],
    )
    .await
});

changeset!(TaskLabelAndTitle, "004-task-label-and-title", |manager| {
    exec(manager, &[
        "ALTER TABLE task ADD COLUMN title VARCHAR(200)",
        "UPDATE task SET title = name WHERE title IS NULL",
        "ALTER TABLE task RENAME COLUMN name TO label",
    ])
    .await?;
    normalise_labels(manager, "task").await?;
    rebuild(
        manager,
        "task",
        "id TEXT NOT NULL CONSTRAINT pk_task PRIMARY KEY,
         label VARCHAR(160) NOT NULL,
         status VARCHAR(20) NOT NULL,
         description VARCHAR(100000),
         project_id TEXT NOT NULL,
         created_at TEXT NOT NULL,
         updated_at TEXT NOT NULL,
         title VARCHAR(200) NOT NULL,
         CONSTRAINT fk_task_project FOREIGN KEY (project_id) REFERENCES project (id) ON DELETE CASCADE,
         CONSTRAINT uq_task_project_label UNIQUE (project_id, label)",
        "id, label, status, description, project_id, created_at, updated_at, title",
        &[],
    )
    .await
});

#[cfg(test)]
mod tests {
    use super::legacy_label;

    #[test]
    fn the_migrations_normalising_expression_turns_a_legacy_name_into_a_label() {
        for (legacy, expected) in [
            ("Vega", "vega"),
            ("Progetto Vega", "progetto-vega"),
            ("../../etc", "etc"),
            ("a/b/c", "a-b-c"),
        ] {
            assert_eq!(legacy_label(legacy), expected);
        }
    }
}
