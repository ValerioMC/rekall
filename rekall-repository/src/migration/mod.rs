//! The schema, built the way `db.changelog-master.yaml` built it: one migration per changeset,
//! named by the changeset's id, applied in the changelog's order. Each is the SQLite spelling of
//! what Liquibase did to H2. Where SQLite cannot alter a table in place (adding or dropping a
//! constraint, changing a default, dropping a column a constraint names), the table is rebuilt:
//! created afresh under a temporary name, filled from the old one, and swapped in.
//!
//! Types: `uuid` and `timestamp` are TEXT (an `Id` and an `Instant` are stored as their text),
//! `varchar(n)` is `VARCHAR(n)` (SQLite does not enforce the length, so the model does, see
//! `rekall_model::constraints`), `boolean` is 0/1, `int` is INTEGER, `clob` is TEXT.

use sea_orm::{ConnectionTrait, DbErr};
use sea_orm_migration::prelude::*;

mod m001_core;
mod m002_documents_on_many_tasks;
mod m003_company;
mod m004_label_title_description;
mod m005_to_m010;
mod m011_to_m018;
mod m019_to_m026;

pub use m004_label_title_description::legacy_label;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m001_core::Environment),
            Box::new(m001_core::Project),
            Box::new(m001_core::Task),
            Box::new(m001_core::Document),
            Box::new(m002_documents_on_many_tasks::DocumentTask),
            Box::new(m002_documents_on_many_tasks::CarryTaskNotes),
            Box::new(m002_documents_on_many_tasks::DropOwnerlessNotes),
            Box::new(m002_documents_on_many_tasks::DocumentLosesItsOwners),
            Box::new(m002_documents_on_many_tasks::DropEnvironment),
            Box::new(m003_company::Company),
            Box::new(m003_company::HoldingCompanyForExistingProjects),
            Box::new(m003_company::ProjectBelongsToACompany),
            Box::new(m003_company::ProjectNameUniquePerCompany),
            Box::new(m004_label_title_description::ProjectLabelAndTitle),
            Box::new(m004_label_title_description::TaskLabelAndTitle),
            Box::new(m005_to_m010::Wrapup),
            Box::new(m005_to_m010::TimeEntry),
            Box::new(m005_to_m010::ProjectBlueprint),
            Box::new(m005_to_m010::ProjectRepoFolder),
            Box::new(m005_to_m010::TaskStep),
            Box::new(m005_to_m010::TaskStepState),
            Box::new(m011_to_m018::ClaudeSession),
            Box::new(m011_to_m018::ClaudeSessionModel),
            Box::new(m011_to_m018::ClaudeSessionEffort),
            Box::new(m011_to_m018::TaskReviewState),
            Box::new(m011_to_m018::TaskWrapupDirective),
            Box::new(m011_to_m018::TaskStepDraft),
            Box::new(m011_to_m018::DropClaudeSession),
            Box::new(m011_to_m018::DropTaskWrapupDirective),
            Box::new(m019_to_m026::CommitReference),
            Box::new(m019_to_m026::CommitReferenceDiff),
            Box::new(m019_to_m026::CommitReferenceInContext),
            Box::new(m019_to_m026::ProjectAutoCommit),
            Box::new(m019_to_m026::Tag),
            Box::new(m019_to_m026::TaskRevision),
            Box::new(m019_to_m026::DocumentContextMode),
            Box::new(m019_to_m026::RunQueue),
        ]
    }
}

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
