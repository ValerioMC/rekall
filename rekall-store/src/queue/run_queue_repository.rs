use rekall_common::RekallError;
use crate::{run_queue, run_queue_item};
use sea_orm::{ConnectionTrait, EntityTrait, QueryOrder};

/// `RunQueueRepository.findFirstByOrderByCreatedAtAsc`.
pub async fn find_oldest(db: &impl ConnectionTrait) -> Result<Option<run_queue::Model>, RekallError> {
    Ok(run_queue::Entity::find()
        .order_by_asc(run_queue::Column::CreatedAt)
        .one(db)
        .await?)
}

/// `RunQueueItemRepository.findAllByOrderByPositionAsc`.
pub async fn list_items_in_order(db: &impl ConnectionTrait) -> Result<Vec<run_queue_item::Model>, RekallError> {
    Ok(run_queue_item::Entity::find()
        .order_by_asc(run_queue_item::Column::Position)
        .all(db)
        .await?)
}
