//! What a person does to a project's exports: write one, list them, download one, delete some.
//! A refused request is not an export and leaves no record; a failure to write or keep the
//! document is recorded as a `FAILED` export so the Exports page shows it. A picture that cannot
//! be printed fails no export: the document says it is missing.

use std::collections::HashSet;

use rekall_core::{in_read, in_write, Ctx, Tx};
use rekall_export::{DocxWriter, ExportPicture, PdfWriter};
use rekall_kernel::{Id, RekallError, Result};
use rekall_store::export::ProjectExportListing;
use rekall_store::project_export;
use rekall_store::repository as repo;
use rekall_store::{project, project_component, project_section, ExportFormat, ExportStatus};

use crate::export::rule::export_failure::ExportFailure;
use crate::export::rule::export_file_name::media_type_of;
use crate::export::rule::export_plan::ExportPlan;
use crate::export::rule::export_selection::ExportSelection;
use crate::export::rule::page_order;
use crate::export::{ExportedFile, ProjectExportClearCommand, ProjectExportCommand, ProjectExportView};
use crate::picture::PictureService;

#[derive(Clone)]
pub struct ProjectExportService {
    ctx: Ctx,
    pictures: PictureService,
}

impl ProjectExportService {
    pub fn new(ctx: Ctx, pictures: PictureService) -> Self {
        Self { ctx, pictures }
    }

    pub async fn export(&self, project_id: Id, command: ProjectExportCommand) -> Result<ProjectExportView> {
        let mut plan = in_read!(&self.ctx, |tx| plan_in(&tx, project_id, &command).await)?;
        plan.document.pictures = self.pictures_of(&plan).await;
        let bytes = match render(plan.format, &plan).await {
            Ok(bytes) => bytes,
            Err(failure) => return Err(self.record_failure(&plan, failure).await),
        };
        match self.store_done(&plan, bytes).await {
            Ok(view) => Ok(view),
            Err(cause) => Err(self.record_failure(&plan, ExportFailure::storing(plan.format, cause.message())).await),
        }
    }

    pub async fn list(&self) -> Result<Vec<ProjectExportView>> {
        in_read!(&self.ctx, |tx| {
            let listing = repo::project_export::list_with_project_and_company(tx.db()).await?;
            Ok::<_, RekallError>(listing.into_iter().map(ProjectExportView::from).collect())
        })
    }

    /// The file of a finished export; a failed one has none.
    pub async fn file(&self, id: Id) -> Result<ExportedFile> {
        in_read!(&self.ctx, |tx| {
            let export = repo::project_export::require(tx.db(), id).await?;
            let bytes = repo::project_export::file_of(tx.db(), id).await?.ok_or_else(|| RekallError::not_found("Export file", id))?;
            Ok::<_, RekallError>(ExportedFile { file_name: export.file_name, media_type: media_type_of(export.format), bytes })
        })
    }

    pub async fn delete(&self, id: Id) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            repo::project_export::require(tx.db(), id).await?;
            repo::project_export::delete(tx.db(), id).await
        })
    }

    /// Deletes every export that matches the command and answers how many went.
    pub async fn clear(&self, command: ProjectExportClearCommand) -> Result<u64> {
        if command.company_id.is_none() && command.project_id.is_none() && command.status.is_none() {
            return Err(RekallError::illegal("Say which exports to clear: those of a company, of a project, or the failed ones."));
        }
        in_write!(&self.ctx, |tx| {
            if let Some(company_id) = command.company_id {
                repo::company::require(tx.db(), company_id).await?;
            }
            if let Some(project_id) = command.project_id {
                repo::project::require(tx.db(), project_id).await?;
            }
            repo::project_export::delete_matching(tx.db(), command.company_id, command.project_id, command.status).await
        })
    }

    /// The pictures the document shows, as PNG. One that cannot be read or drawn is left out.
    async fn pictures_of(&self, plan: &ExportPlan) -> Vec<ExportPicture> {
        let mut pictures = Vec::new();
        for address in plan.document.picture_addresses() {
            match self.pictures.printable(&address).await {
                Ok(printable) => pictures.push(ExportPicture { address, png: printable.png, natural_width: printable.natural_width }),
                Err(failure) => tracing::warn!(%failure, %address, project_id = %plan.project.id, "A picture is missing from an export"),
            }
        }
        pictures
    }

    async fn store_done(&self, plan: &ExportPlan, bytes: Vec<u8>) -> Result<ProjectExportView> {
        in_write!(&self.ctx, |tx| {
            let company = repo::company::require(tx.db(), plan.project.company_id).await?;
            let export = project_export::Model { size_bytes: bytes.len() as i64, ..self.record_of(plan, ExportStatus::Done, None) };
            let stored = repo::project_export::insert_done(tx.db(), export, bytes).await?;
            Ok::<_, RekallError>(ProjectExportView::from(listing_of(stored, &plan.project, company.id, company.name)))
        })
    }

    async fn record_failure(&self, plan: &ExportPlan, failure: ExportFailure) -> RekallError {
        tracing::error!(%failure, project_id = %plan.project.id, format = %plan.format, "An export failed");
        if let Err(recording_failure) = self.insert_failed(plan, &failure).await {
            tracing::error!(%recording_failure, project_id = %plan.project.id, "The failed export could not be recorded");
        }
        failure.into_error()
    }

    async fn insert_failed(&self, plan: &ExportPlan, failure: &ExportFailure) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let export = self.record_of(plan, ExportStatus::Failed, Some(failure));
            repo::project_export::insert_failed(tx.db(), export).await?;
            Ok::<_, RekallError>(())
        })
    }

    fn record_of(&self, plan: &ExportPlan, status: ExportStatus, failure: Option<&ExportFailure>) -> project_export::Model {
        project_export::Model {
            id: Id::random(),
            project_id: plan.project.id,
            format: plan.format,
            status,
            summary: plan.summary.clone(),
            file_name: plan.file_name.clone(),
            size_bytes: 0,
            diagrams_missing: plan.diagrams_missing,
            failure_kind: failure.map(|failure| failure.kind),
            failure_message: failure.map(|failure| failure.message.clone()),
            created_at: self.ctx.now(),
        }
    }
}

fn listing_of(export: project_export::Model, project: &project::Model, company_id: Id, company_name: String) -> ProjectExportListing {
    ProjectExportListing { export, project_label: project.label.clone(), project_title: project.title.clone(), company_id, company_name }
}

async fn plan_in(tx: &Tx, project_id: Id, command: &ProjectExportCommand) -> Result<ExportPlan> {
    let project = repo::project::require(tx.db(), project_id).await?;
    let pages = ticked_pages(tx, project_id, &command.section_ids).await?;
    let components = ticked_components(tx, project_id, &command.component_ids).await?;
    let blueprint_markdown = ticked_blueprint(&project, command.blueprint, !pages.is_empty() || !components.is_empty())?;
    let selection = ExportSelection { blueprint_markdown, pages, components };
    if selection.is_empty() {
        return Err(RekallError::illegal("Tick the blueprint, a page or a component to export."));
    }
    Ok(ExportPlan::of(project, command.format, &selection, &command.diagrams))
}

fn ticked_blueprint(project: &project::Model, ticked: bool, with_other_items: bool) -> Result<Option<String>> {
    if !ticked {
        return Ok(None);
    }
    let text = project.blueprint_markdown.as_deref().filter(|text| !text.trim().is_empty());
    if text.is_none() && !with_other_items {
        return Err(RekallError::illegal("The blueprint is empty. Tick a page or a component, or write the blueprint first."));
    }
    Ok(text.map(str::to_string))
}

async fn ticked_pages(tx: &Tx, project_id: Id, ids: &[Id]) -> Result<Vec<project_section::Model>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let contexts = repo::section_context::list_of_project(tx.db(), project_id).await?;
    let pages = repo::project_section::list_of_project(tx.db(), project_id).await?;
    require_all_of_project(ids, pages.iter().map(|page| page.id), "Page")?;
    let wanted: HashSet<Id> = ids.iter().copied().collect();
    let ticked = pages.into_iter().filter(|page| wanted.contains(&page.id)).collect();
    Ok(page_order::in_project_order(&contexts, ticked))
}

async fn ticked_components(tx: &Tx, project_id: Id, ids: &[Id]) -> Result<Vec<project_component::Model>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let components = repo::project_component::list_of_project(tx.db(), project_id).await?;
    require_all_of_project(ids, components.iter().map(|component| component.id), "Component")?;
    let wanted: HashSet<Id> = ids.iter().copied().collect();
    Ok(components.into_iter().filter(|component| wanted.contains(&component.id)).collect())
}

fn require_all_of_project(wanted: &[Id], known: impl Iterator<Item = Id>, what: &str) -> Result<()> {
    let known: HashSet<Id> = known.collect();
    match wanted.iter().find(|id| !known.contains(id)) {
        Some(unknown) => Err(RekallError::not_found(what, unknown)),
        None => Ok(()),
    }
}

async fn render(format: ExportFormat, plan: &ExportPlan) -> std::result::Result<Vec<u8>, ExportFailure> {
    let document = plan.document.clone();
    let written = tokio::task::spawn_blocking(move || match format {
        ExportFormat::Docx => DocxWriter::write(&document),
        ExportFormat::Pdf => PdfWriter::write(&document),
    })
    .await;
    match written {
        Ok(Ok(bytes)) => Ok(bytes),
        Ok(Err(cause)) => Err(ExportFailure::rendering(format, cause)),
        Err(stopped) => Err(ExportFailure::rendering(format, stopped)),
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/export/service/project_export_service_tests.rs"]
mod tests;
