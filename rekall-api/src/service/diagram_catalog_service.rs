//! Removes a diagram. Console-only, like every other delete, so a session can write a diagram
//! but never take one away.

use rekall_common::{Id, RekallError, Result};
use rekall_model::diagram;
use rekall_repository::repository as repo;
use rekall_service::diagram::DiagramStreamEvent;
use rekall_service::{in_write, DomainEvent, Services};
use sea_orm::EntityTrait;

#[derive(Clone)]
pub struct DiagramCatalogService {
    services: Services,
}

impl DiagramCatalogService {
    pub fn new(services: Services) -> Self {
        Self { services }
    }

    pub async fn delete(&self, id: Id) -> Result<()> {
        in_write!(&self.services.ctx, |tx| {
            repo::diagram::find_by_id(tx.db(), id).await?.ok_or_else(|| RekallError::not_found("Diagram", id))?;
            diagram::Entity::delete_by_id(id).exec(tx.db()).await?;
            tx.publish(DomainEvent::Diagram(DiagramStreamEvent { diagram_id: id, diagram: None, deleted: true }));
            Ok::<_, RekallError>(())
        })
    }
}
