//! Reads the text of every table named by the schema itself, so a column added later is read too.

use rekall_kernel::RekallError;
use sea_orm::{ConnectionTrait, QueryResult, Statement, Value};

const TABLE_NAMES: &str = "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite\\_%' ESCAPE '\\' AND name <> 'seaql_migrations' ORDER BY name";

/// A table of the schema and every column it has.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextTable {
    pub name: String,
    pub columns: Vec<String>,
}

/// Some rows of one table: how many were read, the `rowid` of the last, and the text values among
/// theirs that hold what was asked for.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextPage {
    pub rows: u64,
    pub last_row: Option<i64>,
    pub texts: Vec<String>,
}

pub async fn tables(db: &impl ConnectionTrait) -> Result<Vec<TextTable>, RekallError> {
    let mut tables = Vec::new();
    for row in db.query_all_raw(statement(db, TABLE_NAMES.to_string(), Vec::new())).await? {
        let name: String = row.try_get_by_index(0)?;
        let columns = columns_of(db, &name).await?;
        tables.push(TextTable { name, columns });
    }
    Ok(tables)
}

pub async fn row_count(db: &impl ConnectionTrait, table: &TextTable) -> Result<u64, RekallError> {
    let sql = format!("SELECT COUNT(*) FROM {}", quoted(&table.name));
    let counted = db.query_one_raw(statement(db, sql, Vec::new())).await?;
    let rows: i64 = counted.map(|row| row.try_get_by_index(0)).transpose()?.unwrap_or_default();
    Ok(rows.unsigned_abs())
}

/// At most `limit` rows in `rowid` order, after `after_row` or from the first. A value that is not
/// of SQLite's `text` type, or does not hold `fragment`, is left out.
pub async fn page_holding(
    db: &impl ConnectionTrait,
    table: &TextTable,
    fragment: &str,
    after_row: Option<i64>,
    limit: u32,
) -> Result<TextPage, RekallError> {
    let values: Vec<String> = table.columns.iter().map(|column| held_text(&quoted(column))).collect();
    let after = after_row.map_or("", |_| "WHERE rowid > ?2");
    let sql = format!("SELECT rowid, {} FROM {} {after} ORDER BY rowid LIMIT {limit}", values.join(", "), quoted(&table.name));
    let mut bound = vec![Value::from(fragment)];
    bound.extend(after_row.map(Value::from));
    let rows = db.query_all_raw(statement(db, sql, bound)).await?;
    read_page(&rows, table.columns.len())
}

async fn columns_of(db: &impl ConnectionTrait, table: &str) -> Result<Vec<String>, RekallError> {
    let sql = "SELECT name FROM pragma_table_info(?) ORDER BY cid".to_string();
    let rows = db.query_all_raw(statement(db, sql, vec![Value::from(table)])).await?;
    rows.iter().map(|row| Ok(row.try_get_by_index(0)?)).collect()
}

fn read_page(rows: &[QueryResult], columns: usize) -> Result<TextPage, RekallError> {
    let mut page = TextPage { rows: rows.len() as u64, ..TextPage::default() };
    for row in rows {
        page.last_row = Some(row.try_get_by_index(0)?);
        for column in 1..=columns {
            page.texts.extend(row.try_get_by_index::<Option<String>>(column)?);
        }
    }
    Ok(page)
}

/// `?1` is the fragment, bound once however many columns ask for it.
fn held_text(column: &str) -> String {
    format!("CASE WHEN typeof({column}) = 'text' AND instr({column}, ?1) > 0 THEN {column} END")
}

fn quoted(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

fn statement(db: &impl ConnectionTrait, sql: String, values: Vec<Value>) -> Statement {
    Statement::from_sql_and_values(db.get_database_backend(), sql, values)
}

#[cfg(test)]
#[path = "../../../tests/unit/picture/repository/picture_text_repository_tests.rs"]
mod tests;
