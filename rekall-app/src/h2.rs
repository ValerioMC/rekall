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

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use rekall_common::{Id, Instant};
use rekall_repository::migration::Migrator;
use sea_orm::sqlx::sqlite::SqliteConnectOptions;
use sea_orm::sqlx::sqlite::SqlitePoolOptions;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, SqlxSqliteConnector, Statement, TransactionTrait, Value};
use sea_orm_migration::MigratorTrait;
use tracing::info;

use crate::bootstrap::folder::LEGACY_DATA_FILE_NAME;

const LIQUIBASE_TABLES: [&str; 2] = ["DATABASECHANGELOG", "DATABASECHANGELOGLOCK"];

/// The H2 file an as-yet absent SQLite file would be imported from: `<base>.mv.db` beside
/// `<base>.db`, or `rekall.mv.db` in the same folder.
pub fn legacy_file_beside(sqlite_file: &Path) -> Option<PathBuf> {
    let folder = sqlite_file.parent()?;
    let stem = sqlite_file.file_stem()?.to_string_lossy();
    [folder.join(format!("{stem}.mv.db")), folder.join(LEGACY_DATA_FILE_NAME)].into_iter().find(|p| p.is_file())
}

/// What was carried over, table by table.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ImportReport {
    pub changesets_in_source: usize,
    pub rows: Vec<(String, usize)>,
}

impl ImportReport {
    pub fn total_rows(&self) -> usize {
        self.rows.iter().map(|(_, n)| n).sum()
    }
}

/// The Java runtime and the H2 jar that read the file.
#[derive(Clone, Debug)]
pub struct Importer {
    pub java: PathBuf,
    pub h2_jar: PathBuf,
    pub user: String,
    pub password: String,
}

impl Importer {
    /// `java` from `JAVA_HOME` or the `PATH`; the jar from `REKALL_H2_JAR`, or the newest in the
    /// local Maven repository, where building the Java version put it.
    pub fn locate() -> Result<Self, String> {
        Self::locate_with(None)
    }

    pub fn locate_with(h2_jar: Option<PathBuf>) -> Result<Self, String> {
        let java = match std::env::var_os("JAVA_HOME") {
            Some(home) if Path::new(&home).join("bin/java").is_file() => Path::new(&home).join("bin/java"),
            _ => PathBuf::from("java"),
        };
        if Command::new(&java).arg("-version").output().is_err() {
            return Err("No Java runtime found (set JAVA_HOME or put `java` on the PATH); the H2 file can only be read by H2's own jar.".into());
        }
        let h2_jar = h2_jar
            .or_else(|| std::env::var_os("REKALL_H2_JAR").map(PathBuf::from))
            .or_else(newest_maven_h2_jar)
            .ok_or_else(|| {
                "No H2 jar found. Download h2-<version>.jar from https://repo1.maven.org/maven2/com/h2database/h2/ \
                 (the 2.x version the Java build used) and pass it with --h2-jar or REKALL_H2_JAR."
                    .to_string()
            })?;
        if !h2_jar.is_file() {
            return Err(format!("{} is not a file", h2_jar.display()));
        }
        Ok(Self {
            java,
            h2_jar,
            user: std::env::var("REKALL_DB_USER").unwrap_or_else(|_| "rekall".into()),
            password: std::env::var("REKALL_DB_PASSWORD").unwrap_or_else(|_| "rekall".into()),
        })
    }

    /// Import `h2_file` into a new SQLite file at `target`, which must not exist yet.
    pub fn import_blocking(&self, h2_file: &Path, target: &Path) -> Result<ImportReport, String> {
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|e| e.to_string())?;
        runtime.block_on(self.import(h2_file, target))
    }

    pub async fn import(&self, h2_file: &Path, target: &Path) -> Result<ImportReport, String> {
        if target.exists() {
            return Err(format!("{} already exists; the import only writes a new file.", target.display()));
        }
        if !h2_file.is_file() {
            return Err(format!("{} is not an H2 database file", h2_file.display()));
        }
        let scratch = tempfile::tempdir().map_err(|e| e.to_string())?;
        let export = self.export(h2_file, scratch.path())?;

        let staging = PathBuf::from(format!("{}.importing", target.display()));
        let _ = std::fs::remove_file(&staging);
        let result = load(&export, &staging).await;
        match result {
            Ok(report) => {
                std::fs::rename(&staging, target).map_err(|e| format!("Could not move the import into place: {e}"))?;
                info!("Imported {} rows from {} into {}", report.total_rows(), h2_file.display(), target.display());
                Ok(report)
            }
            Err(failed) => {
                let _ = std::fs::remove_file(&staging);
                for suffix in ["-wal", "-shm"] {
                    let _ = std::fs::remove_file(format!("{}{suffix}", staging.display()));
                }
                Err(failed)
            }
        }
    }

    /// Copy the file aside and have H2 write it out as CSV.
    fn export(&self, h2_file: &Path, scratch: &Path) -> Result<Export, String> {
        let copy_base = scratch.join("source");
        std::fs::copy(h2_file, scratch.join("source.mv.db")).map_err(|e| format!("Could not copy {}: {e}", h2_file.display()))?;
        let url = format!("jdbc:h2:file:{};IFEXISTS=TRUE", copy_base.display());

        let columns_csv = scratch.join("columns.csv");
        self.run_script(
            &url,
            scratch,
            &format!(
                "CALL CSVWRITE('{}', 'SELECT TABLE_NAME, COLUMN_NAME, DATA_TYPE FROM INFORMATION_SCHEMA.COLUMNS \
                 WHERE TABLE_SCHEMA = ''PUBLIC'' ORDER BY TABLE_NAME, ORDINAL_POSITION', 'charset=UTF-8');",
                sql_text(&columns_csv)
            ),
        )?;
        let mut tables: Vec<Table> = Vec::new();
        for row in read_csv(&columns_csv)?.rows {
            let [table, column, data_type] = [0, 1, 2].map(|i| row.get(i).cloned().flatten().unwrap_or_default());
            match tables.last_mut() {
                Some(last) if last.name == table => last.columns.push((column, data_type)),
                _ => tables.push(Table { name: table, columns: vec![(column, data_type)] }),
            }
        }

        let mut script = String::new();
        for table in &tables {
            let select: Vec<String> = table
                .columns
                .iter()
                .map(|(column, data_type)| {
                    if data_type == "TIMESTAMP" {
                        format!("CAST(\"{column}\" AS TIMESTAMP WITH TIME ZONE) AS \"{column}\"")
                    } else {
                        format!("\"{column}\"")
                    }
                })
                .collect();
            let query = format!("SELECT {} FROM \"{}\"", select.join(", "), table.name).replace('\'', "''");
            script.push_str(&format!(
                "CALL CSVWRITE('{}', '{query}', 'charset=UTF-8');\n",
                sql_text(&table_csv(scratch, &table.name))
            ));
        }
        self.run_script(&url, scratch, &script)?;
        Ok(Export { folder: scratch.to_path_buf(), tables })
    }

    fn run_script(&self, url: &str, scratch: &Path, script: &str) -> Result<(), String> {
        let file = scratch.join(format!("script-{}.sql", Id::random()));
        std::fs::write(&file, script).map_err(|e| e.to_string())?;
        let output = Command::new(&self.java)
            .arg("-cp")
            .arg(&self.h2_jar)
            .arg("org.h2.tools.RunScript")
            .args(["-url", url, "-user", &self.user, "-password", &self.password, "-script"])
            .arg(&file)
            .output()
            .map_err(|e| format!("Could not run java: {e}"))?;
        if !output.status.success() {
            let said = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
            return Err(format!("H2 could not read the file: {}", said.trim()));
        }
        Ok(())
    }
}

fn sql_text(path: &Path) -> String {
    path.to_string_lossy().replace('\'', "''")
}

fn table_csv(folder: &Path, table: &str) -> PathBuf {
    folder.join(format!("table-{table}.csv"))
}

fn newest_maven_h2_jar() -> Option<PathBuf> {
    let root = dirs::home_dir()?.join(".m2/repository/com/h2database/h2");
    let mut versions: Vec<(Vec<u64>, PathBuf)> = std::fs::read_dir(&root)
        .ok()?
        .filter_map(|e| e.ok())
        .filter_map(|entry| {
            let version = entry.file_name().to_string_lossy().into_owned();
            let jar = entry.path().join(format!("h2-{version}.jar"));
            let key: Vec<u64> = version.split(['.', '-']).map(|p| p.parse().unwrap_or(0)).collect();
            jar.is_file().then_some((key, jar))
        })
        .collect();
    versions.sort();
    versions.pop().map(|(_, jar)| jar)
}

struct Table {
    name: String,
    columns: Vec<(String, String)>,
}

struct Export {
    folder: PathBuf,
    tables: Vec<Table>,
}

async fn load(export: &Export, staging: &Path) -> Result<ImportReport, String> {
    let options = SqliteConnectOptions::new().filename(staging).create_if_missing(true).foreign_keys(false);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .map_err(|e| e.to_string())?;
    let conn = SqlxSqliteConnector::from_sqlx_sqlite_pool(pool);
    let result = load_into(&conn, export).await;
    let _ = conn.close().await;
    result
}

async fn load_into(conn: &DatabaseConnection, export: &Export) -> Result<ImportReport, String> {
    let applied = applied_changesets(export)?;
    let known: Vec<String> = Migrator::migrations().iter().map(|m| m.name().to_string()).collect();
    if applied.len() > known.len() || applied.iter().zip(&known).any(|(a, k)| a != k) {
        return Err(format!(
            "The H2 file was migrated by changesets this build does not know, or in another order ({}). \
             Import it with the Rekall version that wrote it.",
            applied.join(", ")
        ));
    }
    Migrator::up(conn, Some(applied.len() as u32)).await.map_err(|e| e.to_string())?;

    let mut report = ImportReport { changesets_in_source: applied.len(), rows: Vec::new() };
    let txn = conn.begin().await.map_err(|e| e.to_string())?;
    for table in export.tables.iter().filter(|t| !LIQUIBASE_TABLES.contains(&t.name.as_str())) {
        let target_columns = sqlite_columns(&txn, &table.name).await?;
        if target_columns.is_empty() {
            return Err(format!("The SQLite schema has no table {} at this changeset", table.name));
        }
        let csv = read_csv(&table_csv(&export.folder, &table.name))?;
        let types: HashMap<String, String> = table.columns.iter().cloned().collect();
        let mut chosen: Vec<(usize, String, String)> = Vec::new();
        for (index, header) in csv.header.iter().enumerate() {
            let Some(target) = target_columns.iter().find(|c| c.eq_ignore_ascii_case(header)) else {
                return Err(format!("Column {}.{} has no place in the SQLite schema", table.name, header));
            };
            chosen.push((index, target.clone(), types.get(header).cloned().unwrap_or_default()));
        }
        let sqlite_table = sqlite_table_name(&txn, &table.name).await?;
        let sql = format!(
            "INSERT INTO \"{sqlite_table}\" ({}) VALUES ({})",
            chosen.iter().map(|(_, c, _)| format!("\"{c}\"")).collect::<Vec<_>>().join(", "),
            vec!["?"; chosen.len()].join(", ")
        );
        for row in &csv.rows {
            let mut values = Vec::with_capacity(chosen.len());
            for (index, column, data_type) in &chosen {
                let raw = row.get(*index).cloned().flatten();
                values.push(convert(raw, data_type).map_err(|e| format!("{}.{column}: {e}", table.name))?);
            }
            txn.execute_raw(Statement::from_sql_and_values(DbBackend::Sqlite, &sql, values))
                .await
                .map_err(|e| format!("{}: {e}", table.name))?;
        }
        report.rows.push((sqlite_table, csv.rows.len()));
    }
    txn.commit().await.map_err(|e| e.to_string())?;

    let dangling = conn
        .query_all_raw(Statement::from_string(DbBackend::Sqlite, "PRAGMA foreign_key_check"))
        .await
        .map_err(|e| e.to_string())?;
    if !dangling.is_empty() {
        return Err(format!("{} imported row(s) point at rows that are not there", dangling.len()));
    }
    Migrator::up(conn, None).await.map_err(|e| e.to_string())?;
    Ok(report)
}

fn applied_changesets(export: &Export) -> Result<Vec<String>, String> {
    let log = table_csv(&export.folder, "DATABASECHANGELOG");
    if !log.is_file() {
        return Err("The H2 file has no Liquibase log, so it is not a Rekall database.".into());
    }
    let csv = read_csv(&log)?;
    let column = |name: &str| csv.header.iter().position(|h| h.eq_ignore_ascii_case(name));
    let (Some(id), Some(order)) = (column("ID"), column("ORDEREXECUTED")) else {
        return Err("The Liquibase log has no ID or ORDEREXECUTED column".into());
    };
    let mut entries: Vec<(i64, String)> = csv
        .rows
        .iter()
        .map(|row| {
            let order = row.get(order).cloned().flatten().and_then(|o| o.parse().ok()).unwrap_or(i64::MAX);
            (order, row.get(id).cloned().flatten().unwrap_or_default())
        })
        .collect();
    entries.sort();
    Ok(entries.into_iter().map(|(_, id)| id).collect())
}

async fn sqlite_table_name(conn: &impl ConnectionTrait, h2_name: &str) -> Result<String, String> {
    let rows = conn
        .query_all_raw(Statement::from_string(DbBackend::Sqlite, "SELECT name FROM sqlite_master WHERE type = 'table'"))
        .await
        .map_err(|e| e.to_string())?;
    rows.iter()
        .filter_map(|row| row.try_get_by_index::<String>(0).ok())
        .find(|name| name.eq_ignore_ascii_case(h2_name))
        .ok_or_else(|| format!("The SQLite schema has no table {h2_name}"))
}

async fn sqlite_columns(conn: &impl ConnectionTrait, h2_name: &str) -> Result<Vec<String>, String> {
    let Ok(table) = sqlite_table_name(conn, h2_name).await else { return Ok(Vec::new()) };
    let rows = conn
        .query_all_raw(Statement::from_string(DbBackend::Sqlite, format!("PRAGMA table_info(\"{table}\")")))
        .await
        .map_err(|e| e.to_string())?;
    Ok(rows.iter().filter_map(|row| row.try_get_by_index::<String>(1).ok()).collect())
}

/// An H2 value, as `CSVWRITE` printed it, in the spelling the SQLite schema stores.
fn convert(raw: Option<String>, data_type: &str) -> Result<Value, String> {
    let Some(text) = raw else {
        return Ok(Value::String(None));
    };
    Ok(match data_type {
        "BOOLEAN" => Value::BigInt(Some(match text.to_ascii_uppercase().as_str() {
            "TRUE" => 1,
            "FALSE" => 0,
            other => return Err(format!("not a boolean: {other}")),
        })),
        "INTEGER" | "BIGINT" | "SMALLINT" | "TINYINT" => {
            Value::BigInt(Some(text.trim().parse().map_err(|_| format!("not a number: {text}"))?))
        }
        "TIMESTAMP" | "TIMESTAMP WITH TIME ZONE" => Value::String(Some(parse_h2_timestamp(&text)?.to_db_string())),
        "UUID" => Value::String(Some(text.parse::<Id>().map_err(|_| format!("not a UUID: {text}"))?.to_string())),
        _ => Value::String(Some(text)),
    })
}

/// `2026-01-01 10:00:00.123456+02` (H2 writes a whole-hour offset without its minutes).
fn parse_h2_timestamp(text: &str) -> Result<Instant, String> {
    let trimmed = text.trim();
    let bytes = trimmed.as_bytes();
    let whole_hour_offset = bytes.len() > 3
        && matches!(bytes[bytes.len() - 3], b'+' | b'-')
        && bytes[bytes.len() - 2..].iter().all(u8::is_ascii_digit);
    let normalised = if whole_hour_offset { format!("{trimmed}:00") } else { trimmed.to_string() };
    Instant::parse(&normalised).map_err(|_| format!("not a timestamp: {text}"))
}

struct Csv {
    header: Vec<String>,
    rows: Vec<Vec<Option<String>>>,
}

/// H2's `CSVWRITE` output: every value in double quotes (doubled inside), a NULL as nothing at
/// all, rows on lines of their own, line breaks kept inside quoted values.
fn read_csv(file: &Path) -> Result<Csv, String> {
    let text = std::fs::read_to_string(file).map_err(|e| format!("Could not read {}: {e}", file.display()))?;
    let mut records = parse_csv(&text).into_iter();
    let header = records
        .next()
        .ok_or_else(|| format!("{} is empty", file.display()))?
        .into_iter()
        .map(Option::unwrap_or_default)
        .collect();
    Ok(Csv { header, rows: records.collect() })
}

fn parse_csv(text: &str) -> Vec<Vec<Option<String>>> {
    let mut records = Vec::new();
    let mut record: Vec<Option<String>> = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut in_quotes = false;
    let mut chars = text.chars().peekable();
    let mut pending = false;
    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    field.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            } else {
                field.push(c);
            }
            continue;
        }
        match c {
            '"' => {
                in_quotes = true;
                quoted = true;
                pending = true;
            }
            ',' => {
                record.push(if quoted || !field.is_empty() { Some(std::mem::take(&mut field)) } else { None });
                quoted = false;
                pending = true;
            }
            '\r' => {}
            '\n' => {
                if pending || !record.is_empty() || !field.is_empty() {
                    record.push(if quoted || !field.is_empty() { Some(std::mem::take(&mut field)) } else { None });
                    records.push(std::mem::take(&mut record));
                }
                quoted = false;
                pending = false;
            }
            other => {
                field.push(other);
                pending = true;
            }
        }
    }
    if pending || !record.is_empty() || !field.is_empty() {
        record.push(if quoted || !field.is_empty() { Some(field) } else { None });
        records.push(record);
    }
    records
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csvwrite_output_reads_back_with_nulls_empty_strings_quotes_and_line_breaks() {
        let rows = parse_csv("\"ID\",\"NAME\",\"NOTE\"\n\"1\",\"say \"\"hi\"\"\",\n\"2\",\"\",\"two\nlines\"\n");
        assert_eq!(rows[0], vec![Some("ID".into()), Some("NAME".into()), Some("NOTE".into())]);
        assert_eq!(rows[1], vec![Some("1".into()), Some("say \"hi\"".into()), None]);
        assert_eq!(rows[2], vec![Some("2".into()), Some(String::new()), Some("two\nlines".into())]);
        assert_eq!(rows.len(), 3);
    }

    #[test]
    fn h2_values_take_the_spelling_the_sqlite_schema_stores() {
        assert_eq!(convert(Some("TRUE".into()), "BOOLEAN").unwrap(), Value::BigInt(Some(1)));
        assert_eq!(convert(None, "BOOLEAN").unwrap(), Value::String(None));
        assert_eq!(
            convert(Some("2026-03-01 10:15:30.5+02".into()), "TIMESTAMP").unwrap(),
            Value::String(Some("2026-03-01T08:15:30.500000Z".into()))
        );
        assert_eq!(
            convert(Some("2026-03-01 10:15:30+05:30".into()), "TIMESTAMP WITH TIME ZONE").unwrap(),
            Value::String(Some("2026-03-01T04:45:30.000000Z".into()))
        );
        assert_eq!(
            convert(Some("0F8FAD5B-D9CB-469F-A165-70867728950E".into()), "UUID").unwrap(),
            Value::String(Some("0f8fad5b-d9cb-469f-a165-70867728950e".into()))
        );
        assert_eq!(convert(Some("42".into()), "INTEGER").unwrap(), Value::BigInt(Some(42)));
    }

    #[test]
    fn the_h2_file_beside_a_missing_sqlite_one_is_found() {
        let folder = tempfile::tempdir().unwrap();
        assert_eq!(legacy_file_beside(&folder.path().join("rekall.db")), None);
        std::fs::write(folder.path().join("rekall.mv.db"), "H:2").unwrap();
        assert_eq!(legacy_file_beside(&folder.path().join("rekall.db")), Some(folder.path().join("rekall.mv.db")));
    }
}
