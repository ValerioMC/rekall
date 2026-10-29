use rekall_common::{Id, Instant, RekallError};
use crate::company_logo_image::{Column, Entity, Model};
use sea_orm::sea_query::OnConflict;
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QuerySelect, Set};

pub async fn find_of_company(db: &impl ConnectionTrait, company_id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find_by_id(company_id).one(db).await?)
}

/// Which companies have a logo and its version, without reading the bytes.
pub async fn find_all_versions(db: &impl ConnectionTrait) -> Result<Vec<(Id, Instant)>, RekallError> {
    Ok(Entity::find().select_only().column(Column::CompanyId).column(Column::UpdatedAt).into_tuple().all(db).await?)
}

/// Inserts the company's logo, or replaces the one it has.
pub async fn upsert(db: &impl ConnectionTrait, company_id: Id, png: Vec<u8>, at: Instant) -> Result<(), RekallError> {
    let row = crate::company_logo_image::ActiveModel { company_id: Set(company_id), png: Set(png), updated_at: Set(at) };
    Entity::insert(row)
        .on_conflict(OnConflict::column(Column::CompanyId).update_columns([Column::Png, Column::UpdatedAt]).to_owned())
        .exec(db)
        .await?;
    Ok(())
}

pub async fn delete_by_company_id(db: &impl ConnectionTrait, company_id: Id) -> Result<(), RekallError> {
    Entity::delete_many().filter(Column::CompanyId.eq(company_id)).exec(db).await?;
    Ok(())
}
