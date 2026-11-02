use rekall_kernel::{Id, RekallError};
use crate::connection_company;
use crate::external_connection::{Column, Entity, Model};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder};

pub async fn find_by_id(db: &impl ConnectionTrait, id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find_by_id(id).one(db).await?)
}

/// A lookup that must exist: the connection, or a 404.
pub async fn require(db: &impl ConnectionTrait, id: Id) -> Result<Model, RekallError> {
    find_by_id(db, id).await?.ok_or_else(|| RekallError::not_found("Connection", id))
}

/// Every connection of the workspace, by label.
pub async fn list_all(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().order_by_asc(Column::Label).all(db).await?)
}

/// The connections shared with the company, by label.
pub async fn list_of_company(db: &impl ConnectionTrait, company_id: Id) -> Result<Vec<Model>, RekallError> {
    let links = connection_company::Entity::find().filter(connection_company::Column::CompanyId.eq(company_id)).all(db).await?;
    if links.is_empty() {
        return Ok(Vec::new());
    }
    let connection_ids: Vec<Id> = links.into_iter().map(|link| link.connection_id).collect();
    Ok(Entity::find().filter(Column::Id.is_in(connection_ids)).order_by_asc(Column::Label).all(db).await?)
}

/// The connection under `label`, ignoring case. Labels are stored lowercase.
pub async fn find_by_label(db: &impl ConnectionTrait, label: &str) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find().filter(Column::Label.eq(label.to_lowercase())).one(db).await?)
}

/// Whether a connection other than `except` already uses this label.
pub async fn label_taken(db: &impl ConnectionTrait, label: &str, except: Option<Id>) -> Result<bool, RekallError> {
    let mut query = Entity::find().filter(Column::Label.eq(label.to_lowercase()));
    if let Some(except) = except {
        query = query.filter(Column::Id.ne(except));
    }
    Ok(query.one(db).await?.is_some())
}
