use rekall_common::{Instant, RekallError};
use rekall_model::Stamped;
use sea_orm::{ActiveModelBehavior, ActiveModelTrait, ConnectionTrait, EntityTrait, IntoActiveModel};

/// Writes `after` over `before` only when a value differs: stamped, validated, every column set.
/// An unchanged row keeps its `updated_at`, so an idle save never reorders a list by recency.
pub async fn update_if_changed<M, A>(db: &impl ConnectionTrait, before: &M, mut after: M, now: Instant) -> Result<M, RekallError>
where
    M: Stamped + PartialEq + Clone + IntoActiveModel<A>,
    A: ActiveModelTrait + ActiveModelBehavior + Send,
    <A::Entity as EntityTrait>::Model: IntoActiveModel<A>,
{
    if &after == before {
        return Ok(after);
    }
    after.stamp(now);
    after.validate_update()?;
    after.clone().into_active_model().reset_all().update(db).await?;
    Ok(after)
}
