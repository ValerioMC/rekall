//! Notes: written, attached to and detached from tasks, and deleted, from the console.

use rekall_common::{Id, RekallError, Result};
use rekall_model::constraints::Phase;
use rekall_model::{document, document_task, DocumentContextMode};
use rekall_repository::repository as repo;
use rekall_service::{in_read, in_write, Services, Tx};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter};

use super::catalog::require_task;
use super::Snapshot;
use crate::dto::{DocumentRequest, DocumentResponse};

#[derive(Clone)]
pub struct DocumentService {
    services: Services,
}

impl DocumentService {
    pub fn new(services: Services) -> Self {
        Self { services }
    }

    fn ctx(&self) -> &rekall_service::Ctx {
        &self.services.ctx
    }

    pub async fn list(&self, task_id: Option<Id>, project_id: Option<Id>) -> Result<Vec<DocumentResponse>> {
        in_read!(self.ctx(), |tx| {
            let found = if let Some(task_id) = task_id {
                repo::document::find_by_tasks_id_order_by_title_asc(tx.db(), task_id).await?
            } else if let Some(project_id) = project_id {
                repo::document::find_by_project(tx.db(), project_id).await?
            } else {
                repo::document::find_all_by_order_by_updated_at_desc(tx.db()).await?
            };
            responses(&tx, &found).await
        })
    }

    pub async fn search(&self, term: Option<&str>) -> Result<Vec<DocumentResponse>> {
        let Some(term) = term.filter(|t| !rekall_common::jstr::is_blank(t)) else {
            return Ok(Vec::new());
        };
        in_read!(self.ctx(), |tx| {
            let found = repo::document::search(tx.db(), rekall_common::jstr::trim(term)).await?;
            responses(&tx, &found).await
        })
    }

    pub async fn create(&self, request: DocumentRequest) -> Result<DocumentResponse> {
        in_write!(self.ctx(), |tx| {
            let now = self.ctx().now();
            let mut document = document::Model {
                id: Id::random(),
                title: request.title.clone().unwrap_or_default(),
                kind: request.kind.clone().unwrap_or_default(),
                body_markdown: body(&request),
                source_path: None,
                context_mode: DocumentContextMode::Full,
                created_at: now,
                updated_at: now,
            };
            if let Some(mode) = request.context_mode {
                document.context_mode = mode;
            }
            let wanted = resolve(&tx, request.task_ids.as_deref()).await?;
            document.validate(Phase::Persist)?;
            document.clone().into_active_model().insert(tx.db()).await?;
            link(&tx, document.id, &wanted).await?;
            // A note just created still holds the `LinkedHashSet` it was built with.
            let snapshot = Snapshot::load(tx.db()).await?;
            let mut tasks = Vec::new();
            for link in repo::document::links_of_document_as_added(tx.db(), document.id).await? {
                if let Some(task) = snapshot.tasks.get(&link.task_id) {
                    tasks.push(snapshot.task_ref(task)?);
                }
            }
            Ok::<_, RekallError>(DocumentResponse::of(&document, tasks))
        })
    }

    pub async fn update(&self, id: Id, request: DocumentRequest) -> Result<DocumentResponse> {
        in_write!(self.ctx(), |tx| {
            let before = require(&tx, id).await?;
            let mut document = before.clone();
            document.title = request.title.clone().unwrap_or_default();
            document.kind = request.kind.clone().unwrap_or_default();
            document.body_markdown = body(&request);
            if let Some(mode) = request.context_mode {
                document.context_mode = mode;
            }
            let wanted = resolve(&tx, request.task_ids.as_deref()).await?;
            link(&tx, document.id, &wanted).await?;
            if document != before {
                document.updated_at = self.ctx().now();
                document.validate(Phase::Update)?;
                document.clone().into_active_model().reset_all().update(tx.db()).await?;
            }
            response(&tx, &document).await
        })
    }

    pub async fn delete(&self, id: Id) -> Result<()> {
        in_write!(self.ctx(), |tx| {
            let document = require(&tx, id).await?;
            for link in repo::document::links_of_document(tx.db(), document.id).await? {
                detach(&tx, document.id, link.task_id).await?;
            }
            document::Entity::delete_by_id(document.id).exec(tx.db()).await?;
            Ok::<_, RekallError>(())
        })
    }
}

fn body(request: &DocumentRequest) -> String {
    request.body_markdown.clone().unwrap_or_default()
}

async fn require(tx: &Tx, id: Id) -> Result<document::Model> {
    repo::document::find_by_id(tx.db(), id).await?.ok_or_else(|| RekallError::not_found("Document", id))
}

/// The tasks a note is to be on, each required to exist. At least one: a note on no task is
/// unreachable, and no screen could show it again.
async fn resolve(tx: &Tx, task_ids: Option<&[Id]>) -> Result<Vec<Id>> {
    let Some(task_ids) = task_ids.filter(|ids| !ids.is_empty()) else {
        return Err(RekallError::conflict("A note has to be attached to at least one task"));
    };
    let mut resolved = Vec::new();
    for id in task_ids {
        let task = require_task(tx, *id).await?;
        if !resolved.contains(&task.id) {
            resolved.push(task.id);
        }
    }
    Ok(resolved)
}

/// Detach from every task no longer wanted, then attach to every wanted task it is not on yet,
/// at the end of that task's notes.
async fn link(tx: &Tx, document_id: Id, wanted: &[Id]) -> Result<()> {
    for link in repo::document::links_of_document(tx.db(), document_id).await? {
        if !wanted.contains(&link.task_id) {
            detach(tx, document_id, link.task_id).await?;
        }
    }
    let current: Vec<Id> = repo::document::links_of_document(tx.db(), document_id)
        .await?
        .into_iter()
        .map(|l| l.task_id)
        .collect();
    for task_id in wanted {
        if !current.contains(task_id) {
            let position = repo::document::links_of_task(tx.db(), *task_id).await?.len() as i32;
            document_task::Model { document_id, task_id: *task_id, position }
                .into_active_model()
                .insert(tx.db())
                .await?;
        }
    }
    Ok(())
}

/// `Task.detach`: the link goes, and the notes after it on that task move up one place, as the
/// `@OrderColumn` was rewritten.
pub(crate) async fn detach(tx: &Tx, document_id: Id, task_id: Id) -> Result<()> {
    document_task::Entity::delete_many()
        .filter(document_task::Column::DocumentId.eq(document_id))
        .filter(document_task::Column::TaskId.eq(task_id))
        .exec(tx.db())
        .await?;
    for (at, link) in repo::document::links_of_task(tx.db(), task_id).await?.into_iter().enumerate() {
        if link.position != at as i32 {
            let mut moved = link.clone();
            moved.position = at as i32;
            moved.into_active_model().reset_all().update(tx.db()).await?;
        }
    }
    Ok(())
}

async fn response(tx: &Tx, document: &document::Model) -> Result<DocumentResponse> {
    Ok(responses(tx, std::slice::from_ref(document)).await?.remove(0))
}

async fn responses(tx: &Tx, documents: &[document::Model]) -> Result<Vec<DocumentResponse>> {
    if documents.is_empty() {
        return Ok(Vec::new());
    }
    let snapshot = Snapshot::load(tx.db()).await?;
    let mut out = Vec::with_capacity(documents.len());
    for document in documents {
        let mut tasks = Vec::new();
        for link in repo::document::links_of_document(tx.db(), document.id).await? {
            if let Some(task) = snapshot.tasks.get(&link.task_id) {
                tasks.push(snapshot.task_ref(task)?);
            }
        }
        out.push(DocumentResponse::of(document, tasks));
    }
    Ok(out)
}
