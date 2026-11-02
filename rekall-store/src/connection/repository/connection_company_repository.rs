use rekall_kernel::{Id, RekallError};
use crate::connection_company::{Column, Entity, Model};
use crate::company;
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder};

/// The companies the connection is shared with, by name.
pub async fn companies_of_connection(db: &impl ConnectionTrait, connection_id: Id) -> Result<Vec<company::Model>, RekallError> {
    let company_ids = company_ids_of_connection(db, connection_id).await?;
    if company_ids.is_empty() {
        return Ok(Vec::new());
    }
    Ok(company::Entity::find().filter(company::Column::Id.is_in(company_ids)).order_by_asc(company::Column::Name).all(db).await?)
}

pub async fn company_ids_of_connection(db: &impl ConnectionTrait, connection_id: Id) -> Result<Vec<Id>, RekallError> {
    let links = Entity::find().filter(Column::ConnectionId.eq(connection_id)).all(db).await?;
    Ok(links.into_iter().map(|link| link.company_id).collect())
}

/// Whether a project of this company may bind the connection.
pub async fn is_associated(db: &impl ConnectionTrait, connection_id: Id, company_id: Id) -> Result<bool, RekallError> {
    Ok(Entity::find_by_id((connection_id, company_id)).one(db).await?.is_some())
}

/// The connection is shared with exactly these companies afterwards. A company named twice counts once.
pub async fn replace_companies(db: &impl ConnectionTrait, connection_id: Id, company_ids: &[Id]) -> Result<(), RekallError> {
    Entity::delete_many().filter(Column::ConnectionId.eq(connection_id)).exec(db).await?;
    let mut distinct: Vec<Id> = Vec::with_capacity(company_ids.len());
    for company_id in company_ids {
        if !distinct.contains(company_id) {
            distinct.push(*company_id);
        }
    }
    for company_id in distinct {
        Model { connection_id, company_id }.into_active_model().insert(db).await?;
    }
    Ok(())
}
