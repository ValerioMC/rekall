use rekall_common::RekallError;
use crate::transcript_scan::{ActiveModel, Column, Entity, Model};
use sea_orm::sea_query::OnConflict;
use sea_orm::{ConnectionTrait, EntityTrait, IntoActiveModel};

pub async fn find(db: &impl ConnectionTrait, session_id: &str) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find_by_id(session_id.to_string()).one(db).await?)
}

pub async fn save(db: &impl ConnectionTrait, scan: Model) -> Result<(), RekallError> {
    let row: ActiveModel = scan.into_active_model();
    Entity::insert(row)
        .on_conflict(OnConflict::column(Column::SessionId).update_columns([Column::ScannedBytes, Column::ScannedAt]).to_owned())
        .exec(db)
        .await?;
    Ok(())
}
