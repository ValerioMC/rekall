use std::path::{Path, PathBuf};

use rekall_common::{jstr, Id, RekallError, Result};
use rekall_diagram::{GraphCodec, GraphValidator, SemanticGraph, TraceIndex};
use rekall_model::constraints::Phase;
use rekall_model::diagram;
use rekall_repository::repository as repo;
use sea_orm::{ActiveModelTrait, ConnectionTrait, IntoActiveModel};

use crate::{in_read, in_write, load, Ctx, DomainEvent};

use super::{
    DiagramDraft, DiagramStreamEvent, DiagramSummaryView, DiagramTraceView, DiagramView, GeneratedDiagram, GeneratedDiagramView,
    ProjectFolder, SourceAudit,
};

/// Reads and writes diagrams. A graph is validated before it is stored, so everything read back
/// satisfies the rules the console and the trace index rely on.
#[derive(Clone)]
pub struct DiagramService {
    ctx: Ctx,
}

impl DiagramService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    /// Reads a graph sent as JSON. A document in another format, or breaking a rule, is refused
    /// with every reason, so the sender can fix them in one pass.
    pub fn read_graph(value: serde_json::Value) -> Result<SemanticGraph> {
        let graph = GraphCodec::from_value(value).map_err(|error| RekallError::illegal(error.to_string()))?;
        GraphValidator::validate(&graph).map_err(|violations| RekallError::illegal(violations.to_string()))?;
        Ok(graph)
    }

    pub async fn list(&self) -> Result<Vec<DiagramSummaryView>> {
        in_read!(&self.ctx, |tx| {
            let rows = repo::diagram::find_all_without_graph_order_by_updated_at_desc(tx.db()).await?;
            Ok::<_, RekallError>(rows.iter().map(DiagramSummaryView::of).collect())
        })
    }

    pub async fn get(&self, id: Id) -> Result<DiagramView> {
        in_read!(&self.ctx, |tx| {
            let row = require(tx.db(), id).await?;
            view_of(&row)
        })
    }

    pub async fn write(&self, draft: DiagramDraft) -> Result<DiagramView> {
        GraphValidator::validate(&draft.graph).map_err(|violations| RekallError::illegal(violations.to_string()))?;
        in_write!(&self.ctx, |tx| {
            repo::project::find_by_id(tx.db(), draft.project_id).await?.ok_or_else(|| RekallError::not_found("Project", draft.project_id))?;
            if let Some(task_id) = draft.task_id {
                let task = repo::task::find_by_id(tx.db(), task_id).await?.ok_or_else(|| RekallError::not_found("Task", task_id))?;
                if task.project_id != draft.project_id {
                    return Err(RekallError::illegal("The task a diagram is generated from has to be on the diagram's project."));
                }
            }
            let now = self.ctx.now();
            let existing = match draft.id {
                Some(id) => Some(require(tx.db(), id).await?),
                None => None,
            };
            if existing.as_ref().is_some_and(|row| row.project_id != draft.project_id) {
                return Err(RekallError::illegal("A diagram stays on the project it was made for."));
            }
            let row = diagram::Model {
                id: existing.as_ref().map_or_else(Id::random, |row| row.id),
                project_id: draft.project_id,
                task_id: draft.task_id,
                title: draft.title.trim().to_string(),
                question: draft.question.trim().to_string(),
                graph_json: GraphCodec::write(&draft.graph),
                node_count: count(draft.graph.nodes.len()),
                edge_count: count(draft.graph.edges.len()),
                created_at: existing.as_ref().map_or(now, |row| row.created_at),
                updated_at: now,
            };
            row.validate(if existing.is_some() { Phase::Update } else { Phase::Persist })?;
            match existing {
                Some(_) => row.clone().into_active_model().reset_all().update(tx.db()).await?,
                None => row.clone().into_active_model().insert(tx.db()).await?,
            };
            tx.publish(DomainEvent::Diagram(DiagramStreamEvent { diagram_id: row.id, diagram: Some(DiagramSummaryView::of(&row)), deleted: false }));
            Ok::<_, RekallError>(DiagramView { summary: DiagramSummaryView::of(&row), graph: draft.graph.clone() })
        })
    }

    /// A session's diagram for a task: stored on the task's project with the task as its origin,
    /// after every cited span is held against the project folder. Without a folder nothing can be.
    pub async fn write_generated(&self, generated: GeneratedDiagram) -> Result<GeneratedDiagramView> {
        let (task, project) = in_read!(&self.ctx, |tx| {
            let task = load::resolve_task(tx.db(), generated.project_label.as_deref(), &generated.task_label).await?;
            let project = load::project_of(tx.db(), &task).await?;
            Ok::<_, RekallError>((task, project))
        })?;
        let folder = project.repo_folder.as_deref().filter(|folder| !jstr::is_blank(folder)).map(|folder| PathBuf::from(jstr::strip(folder)));
        if let Some(folder) = folder.clone() {
            let graph = generated.graph.clone();
            tokio::task::spawn_blocking(move || audit_sources(&folder, &graph))
                .await
                .map_err(|failed| RekallError::internal("IllegalStateException", failed))??;
        }
        let diagram = self
            .write(DiagramDraft {
                id: generated.diagram_id,
                project_id: project.id,
                task_id: Some(task.id),
                title: generated.title,
                question: generated.question,
                graph: generated.graph,
            })
            .await?;
        Ok(GeneratedDiagramView {
            diagram,
            task_anchor: format!("project:{} task:{}", project.label, task.label),
            replaced: generated.diagram_id.is_some(),
            sources_checked: folder.is_some(),
        })
    }

    /// CODE → CONCEPT across a project: every diagram with an element covering `file:line`.
    pub async fn trace(&self, project_id: Id, file: &str, line: u32) -> Result<Vec<DiagramTraceView>> {
        in_read!(&self.ctx, |tx| {
            let mut traces = Vec::new();
            for row in repo::diagram::find_all_by_project_id(tx.db(), project_id).await? {
                let graph = stored_graph(&row)?;
                let hits = TraceIndex::of(&graph).nodes_at(file, line);
                if !hits.is_empty() {
                    traces.push(DiagramTraceView { diagram_id: row.id, title: row.title.clone(), hits });
                }
            }
            Ok::<_, RekallError>(traces)
        })
    }
}

async fn require(db: &impl ConnectionTrait, id: Id) -> Result<diagram::Model> {
    repo::diagram::find_by_id(db, id).await?.ok_or_else(|| RekallError::not_found("Diagram", id))
}

fn view_of(row: &diagram::Model) -> Result<DiagramView> {
    Ok(DiagramView { summary: DiagramSummaryView::of(row), graph: stored_graph(row)? })
}

/// A stored graph passed validation on the way in; failing to read one back is corruption.
fn stored_graph(row: &diagram::Model) -> Result<SemanticGraph> {
    GraphCodec::parse(&row.graph_json).map_err(|error| RekallError::internal("CorruptDiagram", format!("Diagram {}: {error}", row.id)))
}

fn audit_sources(folder: &Path, graph: &SemanticGraph) -> Result<()> {
    let folder = ProjectFolder::open(folder)?;
    SourceAudit::check(&folder, graph).map_err(|violations| RekallError::illegal(violations.to_string()))
}

fn count(length: usize) -> i32 {
    i32::try_from(length).unwrap_or(i32::MAX)
}
