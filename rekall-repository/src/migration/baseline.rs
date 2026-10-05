//! The schema every database starts from: `baseline.sql`, the tables and indexes as release
//! v0.2.x left them. Later changes are migrations of their own on top of it.
//!
//! A file written by v0.2.x recorded the 41 changesets that built that schema one by one.
//! `adopt` replaces that history with the baseline before the migrator runs, so such a file opens
//! as it is. A file that stopped short of the last of them has a schema the baseline cannot
//! describe and is refused.

use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, Statement, TransactionTrait};

use super::{changeset, exec};

const SCHEMA: &str = include_str!("baseline.sql");

const BASELINE_VERSION: &str = "000-baseline";

/// The last changeset the baseline replaces: a file that has it holds exactly the baseline schema.
const LAST_REPLACED_VERSION: &str = "030-note-scope";

changeset!(Baseline, "000-baseline", |manager| {
    let statements = statements_of(SCHEMA);
    let borrowed: Vec<&str> = statements.iter().map(String::as_str).collect();
    exec(manager, &borrowed).await
});

fn statements_of(script: &str) -> Vec<String> {
    let without_comments: String = script
        .lines()
        .filter(|line| !line.trim_start().starts_with("--"))
        .collect::<Vec<_>>()
        .join("\n");
    without_comments
        .split(';')
        .map(str::trim)
        .filter(|statement| !statement.is_empty())
        .map(str::to_string)
        .collect()
}

/// Refuses a file whose history the baseline cannot carry forward, without writing to it.
pub(crate) async fn check(conn: &impl ConnectionTrait) -> Result<(), DbErr> {
    let applied = applied_versions(conn).await?;
    match too_old(&applied) {
        Some(refusal) => Err(refusal),
        None => Ok(()),
    }
}

/// Rewrites the migration history of a v0.2.x file to the baseline. A fresh file, or one already
/// on the baseline, is left alone; a file older than v0.2.x is refused.
pub(crate) async fn adopt(conn: &DatabaseConnection) -> Result<(), DbErr> {
    let txn = conn.begin().await?;
    let applied = applied_versions(&txn).await?;
    if let Some(refusal) = too_old(&applied) {
        return Err(refusal);
    }
    if applied.is_empty() || applied.iter().any(|version| version == BASELINE_VERSION) {
        return txn.commit().await;
    }
    let backend = txn.get_database_backend();
    txn.execute_raw(Statement::from_sql_and_values(
        backend,
        "DELETE FROM seaql_migrations WHERE version <= ?",
        [LAST_REPLACED_VERSION.into()],
    ))
    .await?;
    txn.execute_raw(Statement::from_sql_and_values(
        backend,
        "INSERT INTO seaql_migrations (version, applied_at) VALUES (?, strftime('%s', 'now'))",
        [BASELINE_VERSION.into()],
    ))
    .await?;
    txn.commit().await
}

/// A history that is neither fresh, nor on the baseline, nor through the last replaced changeset.
fn too_old(applied: &[String]) -> Option<DbErr> {
    let carried_forward = applied.is_empty()
        || applied.iter().any(|version| version == BASELINE_VERSION || version == LAST_REPLACED_VERSION);
    if carried_forward {
        return None;
    }
    Some(DbErr::Custom(format!(
        "This database was written by a Rekall older than v0.2.0 (its last migration is {}). \
         Open it once with Rekall v0.2.2 to bring it up to date, then open it here.",
        applied.last().map(String::as_str).unwrap_or("unknown")
    )))
}

/// The versions recorded in `seaql_migrations`, oldest first; none when the table is not there.
async fn applied_versions(conn: &impl ConnectionTrait) -> Result<Vec<String>, DbErr> {
    let backend = conn.get_database_backend();
    let table = conn
        .query_one_raw(Statement::from_string(
            backend,
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'seaql_migrations'",
        ))
        .await?;
    if table.is_none() {
        return Ok(Vec::new());
    }
    let rows = conn
        .query_all_raw(Statement::from_string(backend, "SELECT version FROM seaql_migrations ORDER BY version"))
        .await?;
    rows.iter().map(|row| row.try_get_by_index::<String>(0)).collect()
}
