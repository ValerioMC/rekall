//! A company's uploaded logo: normalised once when it is stored, resized to the exact size on each
//! request. Console-only, like the catalog it decorates.

use rekall_common::{Id, RekallError, Result};
use rekall_repository::repository as repo;
use rekall_service::{in_read, in_write, Ctx};

use super::icon_raster;

#[derive(Clone)]
pub struct CompanyLogoService {
    ctx: Ctx,
}

impl CompanyLogoService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    /// Replaces the company's logo with `upload`, whatever its shape or format.
    pub async fn store(&self, company_id: Id, upload: Vec<u8>) -> Result<()> {
        let png = icon_raster::in_background(move || icon_raster::normalise(&upload)).await?;
        in_write!(&self.ctx, |tx| {
            repo::company::require(tx.db(), company_id).await?;
            repo::company_logo_image::upsert(tx.db(), company_id, png, self.ctx.now()).await
        })
    }

    pub async fn clear(&self, company_id: Id) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            repo::company::require(tx.db(), company_id).await?;
            repo::company_logo_image::delete_by_company_id(tx.db(), company_id).await
        })
    }

    /// The logo as a PNG `pixels` wide and tall.
    pub async fn render(&self, company_id: Id, pixels: u32) -> Result<Vec<u8>> {
        let master = in_read!(&self.ctx, |tx| {
            repo::company_logo_image::find_of_company(tx.db(), company_id)
                .await?
                .ok_or_else(|| RekallError::not_found("Logo of company", company_id))
        })?;
        icon_raster::in_background(move || icon_raster::render(&master.png, pixels)).await
    }
}

#[cfg(test)]
#[path = "../../tests/unit/catalog/company_logo_service_tests.rs"]
mod tests;
