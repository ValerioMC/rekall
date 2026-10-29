use rekall_common::{Id, RekallError};
use crate::external_connection::{Column, Entity, Model};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder};

pub async fn find_by_id(db: &impl ConnectionTrait, id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find_by_id(id).one(db).await?)
}

/// A lookup that must exist: the connection, or a 404.
pub async fn require(db: &impl ConnectionTrait, id: Id) -> Result<Model, RekallError> {
    find_by_id(db, id).await?.ok_or_else(|| RekallError::not_found("Connection", id))
}

/// A company's connections, by label.
pub async fn list_of_company(db: &impl ConnectionTrait, company_id: Id) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().filter(Column::CompanyId.eq(company_id)).order_by_asc(Column::Label).all(db).await?)
}

/// The company's connection under `label`, ignoring case. Labels are stored lowercase.
pub async fn find_by_label(db: &impl ConnectionTrait, company_id: Id, label: &str) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find()
        .filter(Column::CompanyId.eq(company_id))
        .filter(Column::Label.eq(label.to_lowercase()))
        .one(db)
        .await?)
}

/// Whether a connection of the company other than `except` already uses this label.
pub async fn label_taken(db: &impl ConnectionTrait, company_id: Id, label: &str, except: Option<Id>) -> Result<bool, RekallError> {
    let mut query = Entity::find().filter(Column::CompanyId.eq(company_id)).filter(Column::Label.eq(label.to_lowercase()));
    if let Some(except) = except {
        query = query.filter(Column::Id.ne(except));
    }
    Ok(query.one(db).await?.is_some())
}
