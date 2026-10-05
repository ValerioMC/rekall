use sea_orm::{ConnectionTrait, DbErr};
use sea_orm_migration::prelude::*;

/// One migration: a unit struct named after it, its id as the migration's name, run in a
/// transaction of its own so it is applied whole or not at all.
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
