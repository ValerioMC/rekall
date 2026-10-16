use rekall_common::{Id, RekallError};
use rekall_model::task_skill::{Column, Entity, Model};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder};

/// Every assignment of every task and step, in the order the person ranked them.
pub async fn list_all(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().order_by_asc(Column::TaskId).order_by_asc(Column::Position).all(db).await?)
}

pub async fn list_of_task(db: &impl ConnectionTrait, task_id: Id) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().filter(Column::TaskId.eq(task_id)).order_by_asc(Column::Position).all(db).await?)
}

/// Replaces the skills of the task itself (`step_id` none) or of one step with `skills`, in order.
pub async fn replace(
    db: &impl ConnectionTrait,
    task_id: Id,
    step_id: Option<Id>,
    skills: Vec<Model>,
) -> Result<(), RekallError> {
    let scope = match step_id {
        Some(step_id) => Column::StepId.eq(step_id),
        None => Column::StepId.is_null(),
    };
    Entity::delete_many().filter(Column::TaskId.eq(task_id)).filter(scope).exec(db).await?;
    for skill in skills {
        Entity::insert(skill.into_active_model()).exec(db).await?;
    }
    Ok(())
}
