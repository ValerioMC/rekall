//! A project's uploaded icon: normalised once when it is stored, resized to the exact size on each
//! request. Console-only, like the catalog it decorates.

use rekall_common::{Id, InternalFailure, RekallError, Result};
use rekall_model::project_icon_image;
use rekall_repository::repository as repo;
use rekall_service::{in_read, in_write, Ctx};
use sea_orm::sea_query::OnConflict;
use sea_orm::{EntityTrait, Set};

use super::icon_raster;

#[derive(Clone)]
pub struct ProjectIconService {
    ctx: Ctx,
}

impl ProjectIconService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    /// Replaces the project's picture with `upload`, whatever its shape or format.
    pub async fn store(&self, project_id: Id, upload: Vec<u8>) -> Result<()> {
        let png = tokio::task::spawn_blocking(move || icon_raster::normalise(&upload))
            .await
            .map_err(|error| RekallError::internal(InternalFailure::Background, error))??;
        in_write!(&self.ctx, |tx| {
            repo::project::require(tx.db(), project_id).await?;
            let row = project_icon_image::ActiveModel {
                project_id: Set(project_id),
                png: Set(png.clone()),
                updated_at: Set(self.ctx.now()),
            };
            project_icon_image::Entity::insert(row)
                .on_conflict(
                    OnConflict::column(project_icon_image::Column::ProjectId)
                        .update_columns([project_icon_image::Column::Png, project_icon_image::Column::UpdatedAt])
                        .to_owned(),
                )
                .exec(tx.db())
                .await?;
            Ok::<_, RekallError>(())
        })
    }

    pub async fn clear(&self, project_id: Id) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            repo::project::require(tx.db(), project_id).await?;
            repo::project_icon_image::delete_by_project_id(tx.db(), project_id).await?;
            Ok::<_, RekallError>(())
        })
    }

    /// The picture as a PNG `pixels` wide and tall.
    pub async fn render(&self, project_id: Id, pixels: u32) -> Result<Vec<u8>> {
        let master = in_read!(&self.ctx, |tx| {
            repo::project_icon_image::find_of_project(tx.db(), project_id)
                .await?
                .ok_or_else(|| RekallError::not_found("Icon image of project", project_id))
        })?;
        tokio::task::spawn_blocking(move || icon_raster::render(&master.png, pixels))
            .await
            .map_err(|error| RekallError::internal(InternalFailure::Background, error))?
    }
}
