use std::collections::HashMap;

use rekall_kernel::{Id, RekallError};
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder};

use crate::export::ProjectExportListing;
use crate::project_export::{Column, Entity, Model};
use crate::enums::ExportStatus;
use crate::{company, project, project_export_file};

pub async fn insert_done(db: &impl ConnectionTrait, export: Model, bytes: Vec<u8>) -> Result<Model, RekallError> {
    refuse_other_status(&export, ExportStatus::Done)?;
    export.validate()?;
    let stored = export.into_active_model().insert(db).await?;
    project_export_file::Model { export_id: stored.id, bytes }.into_active_model().insert(db).await?;
    Ok(stored)
}

pub async fn insert_failed(db: &impl ConnectionTrait, export: Model) -> Result<Model, RekallError> {
    refuse_other_status(&export, ExportStatus::Failed)?;
    export.validate()?;
    Ok(export.into_active_model().insert(db).await?)
}

/// Every export, newest first, with the project and company a page groups it by.
pub async fn list_with_project_and_company(db: &impl ConnectionTrait) -> Result<Vec<ProjectExportListing>, RekallError> {
    let exports = Entity::find().order_by_desc(Column::CreatedAt).all(db).await?;
    let projects: HashMap<Id, project::Model> = project::Entity::find().all(db).await?.into_iter().map(|project| (project.id, project)).collect();
    let companies: HashMap<Id, company::Model> = company::Entity::find().all(db).await?.into_iter().map(|company| (company.id, company)).collect();
    exports
        .into_iter()
        .map(|export| listing_of(export, &projects, &companies))
        .collect()
}

pub async fn find_by_id(db: &impl ConnectionTrait, id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find_by_id(id).one(db).await?)
}

/// A lookup that must exist: the export, or a 404.
pub async fn require(db: &impl ConnectionTrait, id: Id) -> Result<Model, RekallError> {
    find_by_id(db, id).await?.ok_or_else(|| RekallError::not_found("Export", id))
}

/// The file's bytes, absent for a failed export.
pub async fn file_of(db: &impl ConnectionTrait, id: Id) -> Result<Option<Vec<u8>>, RekallError> {
    Ok(project_export_file::Entity::find_by_id(id).one(db).await?.map(|file| file.bytes))
}

pub async fn delete(db: &impl ConnectionTrait, id: Id) -> Result<(), RekallError> {
    Entity::delete_many().filter(Column::Id.eq(id)).exec(db).await?;
    Ok(())
}

fn refuse_other_status(export: &Model, expected: ExportStatus) -> Result<(), RekallError> {
    if export.status == expected {
        return Ok(());
    }
    Err(RekallError::illegal(format!("ProjectExport: this write stores a {expected} export and this one is {}.", export.status)))
}

fn listing_of(export: Model, projects: &HashMap<Id, project::Model>, companies: &HashMap<Id, company::Model>) -> Result<ProjectExportListing, RekallError> {
    let project = projects.get(&export.project_id).ok_or_else(|| RekallError::not_found("Project", export.project_id))?;
    let company = companies.get(&project.company_id).ok_or_else(|| RekallError::not_found("Company", project.company_id))?;
    Ok(ProjectExportListing {
        project_label: project.label.clone(),
        project_title: project.title.clone(),
        company_id: company.id,
        company_name: company.name.clone(),
        export,
    })
}

#[cfg(test)]
#[path = "../../../tests/unit/export/repository/project_export_repository_tests.rs"]
mod tests;
