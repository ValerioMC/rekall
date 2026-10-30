use rekall_kernel::RekallError;
use crate::telemetry_setting::{Entity, Model};
use sea_orm::{ActiveModelTrait, ConnectionTrait, EntityTrait, IntoActiveModel};

/// The one setting row, none until a person first saves it.
pub async fn find(db: &impl ConnectionTrait) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find().one(db).await?)
}

pub async fn insert(db: &impl ConnectionTrait, setting: Model) -> Result<Model, RekallError> {
    setting.validate()?;
    Ok(setting.into_active_model().insert(db).await?)
}
