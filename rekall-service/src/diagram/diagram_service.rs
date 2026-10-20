use std::path::{Path, PathBuf};

use rekall_common::{jstr, Id, InternalFailure, RekallError, Result};
use rekall_diagram::{DiagramDocument, DocumentCodec, DocumentValidator};
use rekall_model::{diagram, project, task};
use rekall_repository::repository as repo;
use sea_orm::{ActiveModelTrait, ConnectionTrait, IntoActiveModel};

use crate::{in_read, in_write, load, Ctx};

use super::{
    DiagramDraft, DiagramSheetView, DiagramStreamEvent, DiagramSummaryView, DiagramTarget, DiagramView, GeneratedDiagram,
    GeneratedDiagramView, ProjectFolder, SourceAudit,
};

/// Reads and writes diagrams. A document is validated before it is stored, so everything read
/// back declares a type on every sheet and links only what its sheets draw.
#[derive(Clone)]
pub struct DiagramService {
    ctx: Ctx,
}

impl DiagramService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    /// Reads a document sent as JSON. One of the wrong shape, or breaking a rule, is refused with
    /// every reason, so the sender can fix them in one pass.
    pub fn read_document(value: serde_json::Value) -> Result<DiagramDocument> {
        let document = DocumentCodec::from_value(value).map_err(|error| RekallError::illegal(error.to_string()))?;
        DocumentValidator::validate(&document).map_err(|violations| RekallError::illegal(violations.to_string()))?;
        Ok(document)
    }

    pub async fn list(&self) -> Result<Vec<DiagramSummaryView>> {
        in_read!(&self.ctx, |tx| {
            let rows = repo::diagram::list_summaries_newest_first(tx.db()).await?;
            rows.iter().map(DiagramSummaryView::of).collect::<Result<Vec<_>>>()
        })
    }

    pub async fn get(&self, id: Id) -> Result<DiagramView> {
        in_read!(&self.ctx, |tx| {
            let row = repo::diagram::require(tx.db(), id).await?;
            view_of(&row, &stored_document(&row)?)
        })
    }

    pub async fn write(&self, draft: DiagramDraft) -> Result<DiagramView> {
        DocumentValidator::validate(&draft.document).map_err(|violations| RekallError::illegal(violations.to_string()))?;
        in_write!(&self.ctx, |tx| {
            repo::project::require(tx.db(), draft.project_id).await?;
            require_task_on_project(tx.db(), draft.task_id, draft.project_id).await?;
            let existing = match draft.id {
                Some(id) => Some(repo::diagram::require(tx.db(), id).await?),
                None => None,
            };
            if existing.as_ref().is_some_and(|row| row.project_id != draft.project_id) {
                return Err(RekallError::illegal("A diagram stays on the project it was made for."));
            }
            let row = self.row_of(&draft, existing.as_ref());
            row.validate()?;
            match existing {
                Some(_) => row.clone().into_active_model().reset_all().update(tx.db()).await?,
                None => row.clone().into_active_model().insert(tx.db()).await?,
            };
            let view = view_of(&row, &draft.document)?;
            tx.publish(DiagramStreamEvent { diagram_id: row.id, diagram: Some(view.summary.clone()), deleted: false });
            Ok::<_, RekallError>(view)
        })
    }

    /// A session's diagram, stored on the project its anchors name, with the task as its origin
    /// when they name one, after every cited span is held against the project folder.
    pub async fn write_generated(&self, generated: GeneratedDiagram) -> Result<GeneratedDiagramView> {
        let (project, task) = in_read!(&self.ctx, |tx| resolve_target(tx.db(), &generated.target).await)?;
        let folder = project.repo_folder.as_deref().filter(|folder| !jstr::is_blank(folder)).map(|folder| PathBuf::from(jstr::strip(folder)));
        if let Some(folder) = folder.clone() {
            let document = generated.document.clone();
            tokio::task::spawn_blocking(move || audit_sources(&folder, &document))
                .await
                .map_err(|failed| RekallError::internal(InternalFailure::Background, failed))??;
        }
        let anchors = task.as_ref().map_or_else(|| project.anchor(), |task| task.anchor(&project.label));
        let diagram = self
            .write(DiagramDraft {
                id: generated.diagram_id,
                project_id: project.id,
                task_id: task.map(|task| task.id),
                title: generated.title,
                question: generated.question,
                document: generated.document,
            })
            .await?;
        Ok(GeneratedDiagramView { diagram, anchors, replaced: generated.diagram_id.is_some(), sources_checked: folder.is_some() })
    }

    /// A replacement keeps the diagram's id and creation time.
    fn row_of(&self, draft: &DiagramDraft, existing: Option<&diagram::Model>) -> diagram::Model {
        let now = self.ctx.now();
        diagram::Model {
            id: existing.map_or_else(Id::random, |row| row.id),
            project_id: draft.project_id,
            task_id: draft.task_id,
            title: draft.title.trim().to_string(),
            question: draft.question.trim().to_string(),
            kinds: DiagramSummaryView::column_of(&draft.document.kinds()),
            walkthrough: draft.document.has_walkthrough(),
            document_json: DocumentCodec::write(&draft.document),
            created_at: existing.map_or(now, |row| row.created_at),
            updated_at: now,
        }
    }
}

async fn require_task_on_project(db: &impl ConnectionTrait, task_id: Option<Id>, project_id: Id) -> Result<()> {
    let Some(task_id) = task_id else { return Ok(()) };
    let task = repo::task::require(db, task_id).await?;
    if task.project_id != project_id {
        return Err(RekallError::illegal("The task a diagram is generated from has to be on the diagram's project."));
    }
    Ok(())
}

async fn resolve_target(db: &impl ConnectionTrait, target: &DiagramTarget) -> Result<(project::Model, Option<task::Model>)> {
    match target {
        DiagramTarget::Project { label } => Ok((load::resolve_project(db, label).await?, None)),
        DiagramTarget::Task { project_label, task_label } => {
            let task = load::resolve_task(db, project_label.as_deref(), task_label).await?;
            Ok((load::project_of(db, &task).await?, Some(task)))
        }
    }
}

fn view_of(row: &diagram::Model, document: &DiagramDocument) -> Result<DiagramView> {
    let sheets = document
        .sheets
        .iter()
        .map(|sheet| DiagramSheetView::of(sheet).ok_or_else(|| corrupt(row, "a sheet declares no known Mermaid type")))
        .collect::<Result<Vec<_>>>()?;
    Ok(DiagramView { summary: DiagramSummaryView::of(row)?, sheets })
}

/// A stored document passed validation on the way in; failing to read one back is corruption.
fn stored_document(row: &diagram::Model) -> Result<DiagramDocument> {
    DocumentCodec::parse(&row.document_json).map_err(|error| corrupt(row, error))
}

fn corrupt(row: &diagram::Model, detail: impl std::fmt::Display) -> RekallError {
    RekallError::internal(InternalFailure::CorruptData, format!("Diagram {}: {detail}", row.id))
}

fn audit_sources(folder: &Path, document: &DiagramDocument) -> Result<()> {
    let folder = ProjectFolder::open(folder)?;
    SourceAudit::check(&folder, document).map_err(|violations| RekallError::illegal(violations.to_string()))
}
