//! The badges a task can carry.

use rekall_common::{jstr, Id, RekallError, Result};
use rekall_model::constraints::Phase;
use rekall_model::tag;
use rekall_repository::repository as repo;
use rekall_service::{in_read, in_write, Services, Tx};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};

use crate::dto::{TagRequest, TagResponse};

#[derive(Clone)]
pub struct TagService {
    services: Services,
}

impl TagService {
    pub fn new(services: Services) -> Self {
        Self { services }
    }

    fn ctx(&self) -> &rekall_service::Ctx {
        &self.services.ctx
    }

    pub async fn list(&self) -> Result<Vec<TagResponse>> {
        in_read!(self.ctx(), |tx| {
            Ok::<_, RekallError>(repo::tag::find_all_by_order_by_name_asc(tx.db()).await?.iter().map(TagResponse::of).collect())
        })
    }

    pub async fn create(&self, request: TagRequest) -> Result<TagResponse> {
        in_write!(self.ctx(), |tx| {
            let name = request.name.clone().unwrap_or_default();
            require_name_free(&tx, &name, None).await?;
            let now = self.ctx().now();
            let tag = tag::Model {
                id: Id::random(),
                name: jstr::trim(&name).to_string(),
                icon: request.icon.clone().unwrap_or_default(),
                color: request.color.clone().unwrap_or_default(),
                created_at: now,
                updated_at: now,
            };
            tag.validate(Phase::Persist)?;
            tag.clone().into_active_model().insert(tx.db()).await?;
            Ok::<_, RekallError>(TagResponse::of(&tag))
        })
    }

    pub async fn update(&self, id: Id, request: TagRequest) -> Result<TagResponse> {
        in_write!(self.ctx(), |tx| {
            let before = require(&tx, id).await?;
            let name = request.name.clone().unwrap_or_default();
            require_name_free(&tx, &name, Some(id)).await?;
            let mut tag = before.clone();
            tag.name = jstr::trim(&name).to_string();
            tag.icon = request.icon.clone().unwrap_or_default();
            tag.color = request.color.clone().unwrap_or_default();
            if tag != before {
                tag.updated_at = self.ctx().now();
                tag.validate(Phase::Update)?;
                tag.clone().into_active_model().reset_all().update(tx.db()).await?;
            }
            Ok::<_, RekallError>(TagResponse::of(&tag))
        })
    }

    pub async fn delete(&self, id: Id) -> Result<()> {
        in_write!(self.ctx(), |tx| {
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
    let taken = match except {
        None => repo::tag::exists_by_name_ignore_case(tx.db(), name).await?,
        Some(id) => repo::tag::exists_by_name_ignore_case_and_id_not(tx.db(), name, id).await?,
    };
    if taken {
        return Err(RekallError::conflict(format!("A tag named \"{name}\" already exists")));
    }
    Ok(())
}
