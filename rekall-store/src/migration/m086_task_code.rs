//! `086`: a project gets a task key and a task a number and a code (`RK-12`). A project's key is
//! suggested from its label, in the order the projects were created, so no two share one; a
//! project's tasks are numbered from 1 in the order they were created.
//!
//! The indexes skip the empty value: a row written without a key or code reads as not assigned
//! yet, and the entities' `validate` refuses it.

use sea_orm::{ConnectionTrait, DbErr, Statement};
use sea_orm_migration::prelude::SchemaManager;
use std::collections::HashSet;

use super::{changeset, exec};
use crate::task_key::TaskKey;

changeset!(TaskCode, "086-task-code", |manager| {
    exec(manager, &[
        "ALTER TABLE project ADD COLUMN task_key VARCHAR(4) NOT NULL DEFAULT ''",
        "ALTER TABLE task ADD COLUMN number INTEGER NOT NULL DEFAULT 0",
        "ALTER TABLE task ADD COLUMN code VARCHAR(12) NOT NULL DEFAULT ''",
    ])
    .await?;
    let keys = give_every_project_a_key(manager).await?;
    for (project_id, key) in &keys {
        number_the_tasks_of(manager, project_id, key).await?;
    }
    exec(manager, &[
        "CREATE UNIQUE INDEX uq_project_task_key ON project (task_key) WHERE task_key <> ''",
        "CREATE UNIQUE INDEX uq_task_code ON task (code) WHERE code <> ''",
        "CREATE UNIQUE INDEX uq_task_number ON task (project_id, number) WHERE number > 0",
    ])
    .await
});

async fn give_every_project_a_key(manager: &SchemaManager<'_>) -> Result<Vec<(String, String)>, DbErr> {
    let connection = manager.get_connection();
    let backend = connection.get_database_backend();
    let rows = connection.query_all_raw(Statement::from_string(backend, "SELECT id, label FROM project ORDER BY created_at, id")).await?;
    let mut taken = HashSet::new();
    let mut keyed = Vec::with_capacity(rows.len());
    for row in &rows {
        let id = row.try_get_by_index::<String>(0)?;
        let key = TaskKey::suggest(&row.try_get_by_index::<String>(1)?, &taken);
        taken.insert(key.clone());
        connection
            .execute_raw(Statement::from_sql_and_values(backend, "UPDATE project SET task_key = ? WHERE id = ?", [key.clone().into(), id.clone().into()]))
            .await?;
        keyed.push((id, key));
    }
    Ok(keyed)
}

async fn number_the_tasks_of(manager: &SchemaManager<'_>, project_id: &str, key: &str) -> Result<(), DbErr> {
    let connection = manager.get_connection();
    let backend = connection.get_database_backend();
    let rows = connection
        .query_all_raw(Statement::from_sql_and_values(backend, "SELECT id FROM task WHERE project_id = ? ORDER BY created_at, id", [project_id.into()]))
        .await?;
    for (at, row) in rows.iter().enumerate() {
        let number = i32::try_from(at + 1).map_err(|failure| DbErr::Custom(failure.to_string()))?;
        connection
            .execute_raw(Statement::from_sql_and_values(
                backend,
                "UPDATE task SET number = ?, code = ? WHERE id = ?",
                [number.into(), TaskKey::code(key, number).into(), row.try_get_by_index::<String>(0)?.into()],
            ))
            .await?;
    }
    Ok(())
}
