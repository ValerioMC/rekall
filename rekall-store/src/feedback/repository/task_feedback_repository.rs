use rekall_kernel::{Id, RekallError};
use crate::enums::{FeedbackKind, FeedbackState};
use crate::task_feedback::{Column, Entity, Model};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder};

/// A task's thread, oldest first.
pub async fn list_of_task(db: &impl ConnectionTrait, task_id: Id) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().filter(Column::TaskId.eq(task_id)).order_by_asc(Column::Seq).all(db).await?)
}

pub async fn find_by_id(db: &impl ConnectionTrait, id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find_by_id(id).one(db).await?)
}

pub async fn find_by_seq(db: &impl ConnectionTrait, task_id: Id, seq: i32) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find().filter(Column::TaskId.eq(task_id)).filter(Column::Seq.eq(seq)).one(db).await?)
}

/// The number the task's next entry takes: one past the highest, 1 for an empty thread.
pub async fn next_seq(db: &impl ConnectionTrait, task_id: Id) -> Result<i32, RekallError> {
    let last = Entity::find().filter(Column::TaskId.eq(task_id)).order_by_desc(Column::Seq).one(db).await?;
    Ok(last.map_or(1, |entry| entry.seq + 1))
}

pub async fn pending_of_task(db: &impl ConnectionTrait, task_id: Id) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find()
        .filter(Column::TaskId.eq(task_id))
        .filter(Column::State.eq(FeedbackState::Pending))
        .order_by_asc(Column::Seq)
        .all(db)
        .await?)
}

/// The revision a further description edit keeps rewriting: the task's `PENDING` one, if any.
pub async fn latest_open_revision(db: &impl ConnectionTrait, task_id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find()
        .filter(Column::TaskId.eq(task_id))
        .filter(Column::Kind.eq(FeedbackKind::Revision))
        .filter(Column::State.eq(FeedbackState::Pending))
        .order_by_desc(Column::Seq)
        .one(db)
        .await?)
}

#[cfg(test)]
#[path = "../../../tests/unit/feedback/repository/task_feedback_repository_tests.rs"]
mod tests;
