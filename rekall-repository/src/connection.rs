//! Opening the database: migrate it on a connection of its own, then hand out a pool.
//!
//! Two settings differ between the two steps, on purpose. The migrations rebuild tables SQLite
//! cannot alter in place (a constraint added or dropped, a default changed), and dropping the old
//! copy of a table with foreign keys enforced would cascade into every row that points at it, so
//! they run with enforcement off and the whole file is checked for dangling references before the
//! pool opens with enforcement on, which is where it stays.
//!
//! The pool uses WAL, so a read never waits for a write, and a busy timeout, so a write that
//! finds another one in progress waits for it instead of failing. Writes are serialised by
//! `BEGIN IMMEDIATE` (see `rekall-service`'s transaction wrapper).

use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use sea_orm::sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, SqlxSqliteConnector, Statement};
use sea_orm_migration::MigratorTrait;

use crate::migration::Migrator;

/// The file name the database has inside the folder a person chose. H2 kept `rekall.mv.db`
/// there; the SQLite file sits beside it under its own name, so a folder that still holds the
/// H2 file can be migrated from it rather than overwritten.
pub const DATA_FILE_NAME: &str = "rekall.db";

const BUSY_TIMEOUT: Duration = Duration::from_secs(10);

/// An open database: the pool, and where it lives when it is a file.
#[derive(Clone, Debug)]
pub struct Database {
    pub conn: DatabaseConnection,
    pub path: Option<PathBuf>,
    /// Keeps a temporary directory alive for as long as a database opened in it is.
    _scratch: Option<Arc<ScratchDir>>,
}

#[derive(Debug)]
struct ScratchDir(PathBuf);

impl Drop for ScratchDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

impl Database {
    pub fn is_file(&self) -> bool {
        self.path.is_some()
    }

    /// Close every pooled connection, so the file can be moved or replaced.
    pub async fn close(self) -> Result<(), DbErr> {
        self.conn.close().await
    }
}

fn options_for(path: &Path) -> SqliteConnectOptions {
    SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .busy_timeout(BUSY_TIMEOUT)
}

/// Open (creating it if absent) the database file at `path`, bringing its schema up to date.
pub async fn open(path: &Path) -> Result<Database, DbErr> {
    migrate(options_for(path)).await?;
    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .min_connections(1)
        .connect_with(options_for(path).foreign_keys(true))
        .await
        .map_err(|e| DbErr::Conn(sea_orm::RuntimeErr::SqlxError(Arc::new(e))))?;
    Ok(Database {
        conn: SqlxSqliteConnector::from_sqlx_sqlite_pool(pool),
        path: Some(path.to_path_buf()),
        _scratch: None,
    })
}

/// A database that lives only as long as the process, which is what runs while no folder has
/// been chosen yet (the setup screen), as `jdbc:h2:mem:rekall-setup` did. One connection holds
/// all of it, so the pool never grows past that one.
pub async fn open_in_memory() -> Result<Database, DbErr> {
    let options = SqliteConnectOptions::from_str("sqlite::memory:")
        .map_err(|e| DbErr::Conn(sea_orm::RuntimeErr::SqlxError(Arc::new(e))))?;
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .min_connections(1)
        .idle_timeout(None)
        .max_lifetime(None)
        .connect_with(options.foreign_keys(false))
        .await
        .map_err(|e| DbErr::Conn(sea_orm::RuntimeErr::SqlxError(Arc::new(e))))?;
    let conn = SqlxSqliteConnector::from_sqlx_sqlite_pool(pool);
    Migrator::up(&conn, None).await?;
    check_foreign_keys(&conn).await?;
    conn.execute_unprepared("PRAGMA foreign_keys = ON").await?;
    Ok(Database { conn, path: None, _scratch: None })
}

/// A file database in a fresh temporary folder, removed when the last handle is dropped: what the
/// tests run on, so they exercise the same pool, WAL and migrations the application does.
pub async fn open_temporary() -> Result<Database, DbErr> {
    let folder = std::env::temp_dir().join(format!("rekall-test-{}", uuid_like()));
    std::fs::create_dir_all(&folder).map_err(|e| DbErr::Custom(e.to_string()))?;
    let mut database = open(&folder.join(DATA_FILE_NAME)).await?;
    database._scratch = Some(Arc::new(ScratchDir(folder)));
    Ok(database)
}

fn uuid_like() -> String {
    rekall_common::Id::random().to_string()
}

async fn migrate(options: SqliteConnectOptions) -> Result<(), DbErr> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options.foreign_keys(false))
        .await
        .map_err(|e| DbErr::Conn(sea_orm::RuntimeErr::SqlxError(Arc::new(e))))?;
    let conn = SqlxSqliteConnector::from_sqlx_sqlite_pool(pool);
    Migrator::up(&conn, None).await?;
    check_foreign_keys(&conn).await?;
    conn.close().await
}

/// Refuse to open a file whose rows point at rows that are not there. The Java schema could not
/// hold one (H2 enforced every key on every write); a SQLite file edited by hand could.
async fn check_foreign_keys(conn: &DatabaseConnection) -> Result<(), DbErr> {
    let dangling = conn
        .query_all_raw(Statement::from_string(conn.get_database_backend(), "PRAGMA foreign_key_check"))
        .await?;
    if dangling.is_empty() {
        return Ok(());
    }
    let tables: Vec<String> = dangling
        .iter()
        .filter_map(|row| row.try_get_by_index::<String>(0).ok())
        .collect();
    Err(DbErr::Custom(format!(
        "The database holds {} row(s) whose references point at nothing, in: {}",
        dangling.len(),
        tables.join(", ")
    )))
}
