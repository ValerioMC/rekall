use rekall_common::{Id, RekallError};
use crate::company;
use crate::project::{Column, Entity, Model};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder};


pub async fn find_by_id(db: &impl ConnectionTrait, id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find_by_id(id).one(db).await?)
}

/// The project by id, or `NotFound` when there is none.
pub async fn require(db: &impl ConnectionTrait, id: Id) -> Result<Model, RekallError> {
    find_by_id(db, id).await?.ok_or_else(|| RekallError::not_found("Project", id))
}

pub async fn find_by_ids(db: &impl ConnectionTrait, ids: Vec<Id>) -> Result<Vec<Model>, RekallError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    Ok(Entity::find().filter(Column::Id.is_in(ids)).all(db).await?)
}

/// Every project, in creation order.
pub async fn list_oldest_first(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().order_by_asc(Column::CreatedAt).order_by_asc(Column::Id).all(db).await?)
}

/// Projects whose label is `label`, ignoring case, oldest first. Labels are stored lowercase, so
/// this is a plain comparison in SQL.
pub async fn find_by_label(db: &impl ConnectionTrait, label: &str) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find()
        .filter(Column::Label.eq(label.to_lowercase()))
        .order_by_asc(Column::CreatedAt)
        .order_by_asc(Column::Id)
        .all(db)
        .await?)
}

/// The project `company:<name> project:<label>` names, both ignoring case.
pub async fn find_by_anchor(db: &impl ConnectionTrait, company_name: &str, label: &str) -> Result<Option<Model>, RekallError> {
    let Some(company) = crate::repository::company::find_by_name(db, company_name).await? else {
        return Ok(None);
    };
    Ok(Entity::find()
        .filter(Column::CompanyId.eq(company.id))
        .filter(Column::Label.eq(label.to_lowercase()))
        .one(db)
        .await?)
}

pub async fn list_of_company(db: &impl ConnectionTrait, company_id: Id) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find()
        .filter(Column::CompanyId.eq(company_id))
        .order_by_asc(Column::Label)
        .all(db)
        .await?)
}

/// By company name, then by label: the order the catalog lists projects in.
pub async fn list_in_catalog_order(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find()
        .inner_join(company::Entity)
        .order_by_asc(company::Column::Name)
        .order_by_asc(Column::Label)
        .all(db)
        .await?)
}

/// Whether a project of the company other than `except` already uses this label.
pub async fn label_taken(db: &impl ConnectionTrait, company_id: Id, label: &str, except: Option<Id>) -> Result<bool, RekallError> {
    let mut query = Entity::find().filter(Column::CompanyId.eq(company_id)).filter(Column::Label.eq(label.to_lowercase()));
    if let Some(except) = except {
        query = query.filter(Column::Id.ne(except));
    }
    Ok(query.one(db).await?.is_some())
}
