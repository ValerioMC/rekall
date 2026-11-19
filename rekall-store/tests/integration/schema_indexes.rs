//! Every foreign-key column leads an index, so deleting a parent does not scan its children.

use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};

async fn names(db: &DatabaseConnection, sql: &str, table: &str) -> Vec<(String, String)> {
    db.query_all_raw(Statement::from_sql_and_values(db.get_database_backend(), sql, [table.into()]))
        .await
        .unwrap()
        .iter()
        .map(|row| (row.try_get_by_index::<String>(0).unwrap(), row.try_get_by_index::<String>(1).unwrap_or_default()))
        .collect()
}

async fn tables(db: &DatabaseConnection) -> Vec<String> {
    db.query_all_raw(Statement::from_string(
        db.get_database_backend(),
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' AND name <> 'seaql_migrations' ORDER BY name",
    ))
    .await
    .unwrap()
    .iter()
    .map(|row| row.try_get_by_index::<String>(0).unwrap())
    .collect()
}

async fn foreign_key_columns(db: &DatabaseConnection, table: &str) -> Vec<String> {
    let first_columns = "SELECT \"from\", \"table\" FROM pragma_foreign_key_list(?) WHERE seq = 0";
    names(db, first_columns, table).await.into_iter().map(|(column, _)| column).collect()
}

async fn leading_index_columns(db: &DatabaseConnection, table: &str) -> Vec<String> {
    let leading = "SELECT info.name, list.name FROM pragma_index_list(?) AS list, pragma_index_info(list.name) AS info WHERE info.seqno = 0";
    names(db, leading, table).await.into_iter().map(|(column, _)| column).collect()
}

#[tokio::test]
async fn every_foreign_key_column_leads_an_index() {
    let database = rekall_store::open_temporary().await.unwrap();
    let db = &database.conn;
    let mut unindexed = Vec::new();
    for table in tables(db).await {
        let leading = leading_index_columns(db, &table).await;
        for column in foreign_key_columns(db, &table).await {
            if !leading.contains(&column) {
                unindexed.push(format!("{table}.{column}"));
            }
        }
    }
    assert!(unindexed.is_empty(), "foreign keys with no index leading with them: {unindexed:?}");
}
