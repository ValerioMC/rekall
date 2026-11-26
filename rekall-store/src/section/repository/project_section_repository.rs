use rekall_kernel::{Id, RekallError};
use crate::project_section::{Column, Entity, Model};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder};

use super::section_context_repository;

pub async fn find_by_id(db: &impl ConnectionTrait, id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find_by_id(id).one(db).await?)
}

/// A lookup that must exist: the section, or a 404.
pub async fn require(db: &impl ConnectionTrait, id: Id) -> Result<Model, RekallError> {
    find_by_id(db, id).await?.ok_or_else(|| RekallError::not_found("Section", id))
}

/// A project's sections in the order the console shows them: its contexts in their order, and
/// inside each the sections in theirs.
pub async fn list_of_project(db: &impl ConnectionTrait, project_id: Id) -> Result<Vec<Model>, RekallError> {
    let contexts = section_context_repository::list_of_project(db, project_id).await?;
    let mut sections = Entity::find()
        .filter(Column::ProjectId.eq(project_id))
        .order_by_asc(Column::Position)
        .order_by_asc(Column::CreatedAt)
        .all(db)
        .await?;
    sections.sort_by_key(|section| contexts.iter().position(|context| context.id == section.context_id).unwrap_or(usize::MAX));
    Ok(sections)
}

/// The sections one context holds, in their order.
pub async fn list_of_context(db: &impl ConnectionTrait, context_id: Id) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find()
        .filter(Column::ContextId.eq(context_id))
        .order_by_asc(Column::Position)
        .order_by_asc(Column::CreatedAt)
        .all(db)
        .await?)
}
