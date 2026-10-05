//! Companies: listed, created, renamed and deleted from the console. Deleting one cascades to its
//! projects and their tasks.

use rekall_common::{Id, RekallError, Result};
use rekall_model::company;
use rekall_repository::repository as repo;
use rekall_service::{in_read, in_write, Ctx};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};

use super::Snapshot;
use crate::dto::{CompanyRequest, CompanyResponse};

#[derive(Clone)]
pub struct CompanyService {
    ctx: Ctx,
}

impl CompanyService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    pub async fn list(&self) -> Result<Vec<CompanyResponse>> {
        in_read!(&self.ctx, |tx| {
            let snapshot = Snapshot::load(tx.db()).await?;
            let companies = repo::company::find_all_by_order_by_name_asc(tx.db()).await?;
            Ok::<_, RekallError>(companies.iter().map(|company| snapshot.company_response(company)).collect())
        })
    }

    pub async fn create(&self, request: CompanyRequest) -> Result<CompanyResponse> {
        in_write!(&self.ctx, |tx| {
            let now = self.ctx.now();
            let company = company::Model {
                id: Id::random(),
                name: request.name.clone().unwrap_or_default(),
                description: request.description.clone(),
                created_at: now,
                updated_at: now,
            };
            company.validate()?;
            company.clone().into_active_model().insert(tx.db()).await?;
            Ok::<_, RekallError>(CompanyResponse::of(&company, 0, 0))
        })
    }

    pub async fn update(&self, id: Id, request: CompanyRequest) -> Result<CompanyResponse> {
        in_write!(&self.ctx, |tx| {
            let before = repo::company::require(tx.db(), id).await?;
            let mut company = before.clone();
            company.name = request.name.clone().unwrap_or_default();
            company.description = request.description.clone();
            let company = repo::update_if_changed(tx.db(), &before, company, self.ctx.now()).await?;
            let snapshot = Snapshot::load(tx.db()).await?;
            Ok::<_, RekallError>(snapshot.company_response(&company))
        })
    }

    pub async fn delete(&self, id: Id) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let company = repo::company::require(tx.db(), id).await?;
            company::Entity::delete_by_id(company.id).exec(tx.db()).await?;
            Ok::<_, RekallError>(())
        })
    }
}
