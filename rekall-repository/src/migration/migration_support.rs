use sea_orm::{ConnectionTrait, DbErr};
use sea_orm_migration::prelude::*;

/// One changeset: a unit struct named after it, its Liquibase id as the migration's name, run in
/// a transaction of its own so a changeset is applied whole or not at all.
macro_rules! changeset {
    ($name:ident, $id:literal, |$manager:ident| $body:block) => {
        pub struct $name;

        impl sea_orm_migration::MigrationName for $name {
            fn name(&self) -> &str {
                $id
            }
        }

        #[async_trait::async_trait]
        impl sea_orm_migration::MigrationTrait for $name {
            fn use_transaction(&self) -> Option<bool> {
                Some(true)
            }

            async fn up(&self, $manager: &sea_orm_migration::SchemaManager) -> Result<(), sea_orm::DbErr> $body
        }
    };
}
pub(crate) use changeset;

/// Run each statement in turn.
pub(crate) async fn exec(manager: &SchemaManager<'_>, statements: &[&str]) -> Result<(), DbErr> {
    let conn = manager.get_connection();
    for statement in statements {
        conn.execute_unprepared(statement).await?;
    }
    Ok(())
}

/// Rebuild `table` with a new definition, SQLite's documented way of changing what `ALTER TABLE`
/// cannot. `definition` is the column and constraint list of the new table; `columns` are copied
/// across by name; `indexes` are created again afterwards, since dropping the old table drops its
/// indexes with it. Runs with foreign key enforcement off (see `connection`), so dropping the old
/// copy deletes nothing that points at it.
pub(crate) async fn rebuild(
    manager: &SchemaManager<'_>,
    table: &str,
    definition: &str,
    columns: &str,
    indexes: &[&str],
) -> Result<(), DbErr> {
    let conn = manager.get_connection();
    let staging = format!("{table}__rebuild");
    conn.execute_unprepared(&format!("CREATE TABLE {staging} ({definition})")).await?;
    conn.execute_unprepared(&format!("INSERT INTO {staging} ({columns}) SELECT {columns} FROM {table}"))
        .await?;
    conn.execute_unprepared(&format!("DROP TABLE {table}")).await?;
    conn.execute_unprepared(&format!("ALTER TABLE {staging} RENAME TO {table}")).await?;
    for index in indexes {
        conn.execute_unprepared(index).await?;
    }
    Ok(())
}

/// What `CURRENT_TIMESTAMP` wrote in a changeset, in the spelling an `Instant` column holds.
pub(crate) const NOW: &str = "(strftime('%Y-%m-%dT%H:%M:%S', 'now') || '.000000Z')";
