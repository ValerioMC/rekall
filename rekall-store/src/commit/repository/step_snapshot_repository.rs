use rekall_kernel::{Id, RekallError};
use crate::step_snapshot::{Column, Entity, Model};
use crate::step_snapshot_file;
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder};

/// The snapshot of a step, or of a task with no steps when `step_id` is `None`.
pub async fn find(db: &impl ConnectionTrait, task_id: Id, step_id: Option<Id>) -> Result<Option<Model>, RekallError> {
    let owner = match step_id {
        Some(step_id) => Condition::all().add(Column::StepId.eq(step_id)),
        None => Condition::all().add(Column::TaskId.eq(task_id)).add(Column::StepId.is_null()),
    };
    Ok(Entity::find().filter(owner).one(db).await?)
}

pub async fn files_of(db: &impl ConnectionTrait, snapshot_id: Id) -> Result<Vec<step_snapshot_file::Model>, RekallError> {
    Ok(step_snapshot_file::Entity::find()
        .filter(step_snapshot_file::Column::SnapshotId.eq(snapshot_id))
        .order_by_asc(step_snapshot_file::Column::Path)
        .all(db)
        .await?)
}

/// The snapshot of the same step or task becomes `snapshot` and `files`; the earlier one goes.
pub async fn replace(
    db: &impl ConnectionTrait,
    snapshot: Model,
    files: Vec<(String, Option<String>)>,
) -> Result<(), RekallError> {
    if let Some(earlier) = find(db, snapshot.task_id, snapshot.step_id).await? {
        Entity::delete_by_id(earlier.id).exec(db).await?;
    }
    let snapshot_id = snapshot.id;
    snapshot.into_active_model().insert(db).await?;
    for (path, content_hash) in files {
        step_snapshot_file::Model { snapshot_id, path, content_hash }.into_active_model().insert(db).await?;
    }
    Ok(())
}
