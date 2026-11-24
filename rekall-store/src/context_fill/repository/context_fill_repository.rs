use rekall_kernel::{Id, RekallError};
use crate::context_fill::{ActiveModel, Column, Entity, Model};
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder, Set};

/// The task's own fill when `step_id` is `None`, else the step's.
pub async fn find(db: &impl ConnectionTrait, task_id: Id, step_id: Option<Id>) -> Result<Option<Model>, RekallError> {
    let owner = match step_id {
        Some(step_id) => Condition::all().add(Column::TaskId.eq(task_id)).add(Column::StepId.eq(step_id)),
        None => Condition::all().add(Column::TaskId.eq(task_id)).add(Column::StepId.is_null()),
    };
    Ok(Entity::find().filter(owner).one(db).await?)
}

/// The task's row and its steps' rows, the task's own first.
pub async fn list_for_task(db: &impl ConnectionTrait, task_id: Id) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find()
        .filter(Column::TaskId.eq(task_id))
        .order_by_asc(Column::StepId)
        .all(db)
        .await?)
}

/// Overwrites the fill of the row's task and step, or inserts it when there is none yet.
pub async fn upsert(db: &impl ConnectionTrait, fill: Model) -> Result<(), RekallError> {
    match find(db, fill.task_id, fill.step_id).await? {
        Some(earlier) => {
            ActiveModel {
                id: Set(earlier.id),
                used_tokens: Set(fill.used_tokens),
                window_tokens: Set(fill.window_tokens),
                recorded_at: Set(fill.recorded_at),
                ..Default::default()
            }
            .update(db)
            .await?;
        }
        None => {
            fill.into_active_model().insert(db).await?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/unit/context_fill/repository/context_fill_repository_tests.rs"]
mod tests;
