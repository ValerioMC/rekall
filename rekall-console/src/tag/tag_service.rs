//! The badges a task can carry.

use rekall_common::{jstr, Id, RekallError, Result};
use rekall_store::tag;
use rekall_store::repository as repo;
use rekall_service::{in_read, in_write, Ctx, Tx};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};

use crate::tag::{TagCommand, TagView};

#[derive(Clone)]
pub struct TagService {
    ctx: Ctx,
}

impl TagService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    pub async fn list(&self) -> Result<Vec<TagView>> {
        in_read!(&self.ctx, |tx| {
            Ok::<_, RekallError>(repo::tag::list_by_name(tx.db()).await?.iter().map(TagView::of).collect())
        })
    }

    pub async fn create(&self, command: TagCommand) -> Result<TagView> {
        in_write!(&self.ctx, |tx| {
            let name = command.name.clone().unwrap_or_default();
            require_name_free(&tx, &name, None).await?;
            let now = self.ctx.now();
            let tag = tag::Model {
                id: Id::random(),
                name: jstr::trim(&name).to_string(),
                icon: command.icon.clone().unwrap_or_default(),
                color: command.color.clone().unwrap_or_default(),
                created_at: now,
                updated_at: now,
            };
            tag.validate()?;
            tag.clone().into_active_model().insert(tx.db()).await?;
            Ok::<_, RekallError>(TagView::of(&tag))
        })
    }

    pub async fn update(&self, id: Id, command: TagCommand) -> Result<TagView> {
        in_write!(&self.ctx, |tx| {
            let before = require(&tx, id).await?;
            let name = command.name.clone().unwrap_or_default();
            require_name_free(&tx, &name, Some(id)).await?;
            let mut tag = before.clone();
            tag.name = jstr::trim(&name).to_string();
            tag.icon = command.icon.clone().unwrap_or_default();
            tag.color = command.color.clone().unwrap_or_default();
            let tag = repo::update_if_changed(tx.db(), &before, tag, self.ctx.now()).await?;
            Ok::<_, RekallError>(TagView::of(&tag))
        })
    }

    pub async fn delete(&self, id: Id) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let tag = require(&tx, id).await?;
            tag::Entity::delete_by_id(tag.id).exec(tx.db()).await?;
            Ok::<_, RekallError>(())
        })
    }
}

async fn require(tx: &Tx, id: Id) -> Result<tag::Model> {
    repo::tag::find_by_id(tx.db(), id).await?.ok_or_else(|| RekallError::not_found("Tag", id))
}

async fn require_name_free(tx: &Tx, name: &str, except: Option<Id>) -> Result<()> {
    let name = jstr::trim(name);
    if repo::tag::name_taken(tx.db(), name, except).await? {
        return Err(RekallError::conflict(format!("A tag named \"{name}\" already exists")));
    }
    Ok(())
}
