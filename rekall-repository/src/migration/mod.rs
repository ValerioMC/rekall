//! The schema, built the way `db.changelog-master.yaml` built it: one migration per changeset,
//! named by the changeset's id, applied in the changelog's order. Each is the SQLite spelling of
//! what Liquibase did to H2. Where SQLite cannot alter a table in place (adding or dropping a
//! constraint, changing a default, dropping a column a constraint names), the table is rebuilt:
//! created afresh under a temporary name, filled from the old one, and swapped in.
//!
//! Types: `uuid` and `timestamp` are TEXT (an `Id` and an `Instant` are stored as their text),
//! `varchar(n)` is `VARCHAR(n)` (SQLite does not enforce the length, so the model does, see
//! `rekall_model::constraints`), `boolean` is 0/1, `int` is INTEGER, `clob` is TEXT.

mod m001_core;
mod m002_documents_on_many_tasks;
mod m003_company;
mod m004_label_title_description;
mod m005_to_m010;
mod m011_to_m018;
mod m019_to_m026;
mod m027_project_icon;
mod m028_diagram;
mod migration_support;
mod migrator;

pub use m004_label_title_description::legacy_label;
pub use migrator::Migrator;

pub(crate) use migration_support::{changeset, exec, rebuild, NOW};
