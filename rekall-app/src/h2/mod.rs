//! Moving a database from the Java build's H2 file (`rekall.mv.db`) into SQLite (`rekall.db`).
//!
//! H2 has no reader outside Java, so the H2 jar does the reading: `org.h2.tools.RunScript` runs
//! `CSVWRITE` over a copy of the file (the original is never opened), one CSV per table plus the
//! column types and the Liquibase log. The import then builds a SQLite schema with exactly the
//! migrations whose changesets the H2 file had applied (the migrations carry the changesets' ids),
//! copies every row in, checks every reference resolves, and runs the migrations that remain, so
//! a file from an older version arrives migrated the way Liquibase would have done it.
//!
//! H2 kept the `timestamp` columns as local time in the JVM's zone; they are read back through
//! `CAST(... AS TIMESTAMP WITH TIME ZONE)` in the same zone, so each arrives as the instant it was.
//! Run the import on the machine (and in the time zone) the Java build ran in.

mod h2_migration;
mod import_report;
mod importer;

pub use h2_migration::migrate_from_h2;
pub use import_report::ImportReport;
pub use importer::{legacy_file_beside, Importer};
