//! Notes: written, scoped, attached to and detached from tasks, and deleted, from the console.

use rekall_common::{Id, RekallError, Result};
use rekall_store::note_scope::NoteScope;
use rekall_store::{document, document_task, DocumentContextMode};
use rekall_store::repository as repo;
use rekall_service::{in_read, in_write, Ctx, Tx};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter};

use crate::catalog::Snapshot;
use crate::note::{DocumentCommand, DocumentView};

#[derive(Clone)]
pub struct DocumentService {
    ctx: Ctx,
}

impl DocumentService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    pub async fn list(&self, task_id: Option<Id>, project_id: Option<Id>) -> Result<Vec<DocumentView>> {
        in_read!(&self.ctx, |tx| {
            let found = if let Some(task_id) = task_id {
                repo::document::notes_of_task_by_title(tx.db(), task_id).await?
            } else if let Some(project_id) = project_id {
                repo::document::list_on_project_tasks(tx.db(), project_id).await?
            } else {
                repo::document::list_newest_first(tx.db()).await?
            };
            responses(&tx, &found).await
        })
    }

    pub async fn search(&self, term: Option<&str>) -> Result<Vec<DocumentView>> {
        let Some(term) = term.filter(|t| !rekall_common::jstr::is_blank(t)) else {
            return Ok(Vec::new());
        };
        in_read!(&self.ctx, |tx| {
            let found = repo::document::search_by_title(tx.db(), rekall_common::jstr::trim(term)).await?;
            responses(&tx, &found).await
        })
    }

    pub async fn create(&self, command: DocumentCommand) -> Result<DocumentView> {
        in_write!(&self.ctx, |tx| {
            let now = self.ctx.now();
            let mut document = document::Model {
                id: Id::random(),
                title: command.title.clone().unwrap_or_default(),
                kind: command.kind.clone().unwrap_or_default(),
                body_markdown: body(&command),
                source_path: None,
                context_mode: DocumentContextMode::Full,
                scope_company_id: None,
                scope_project_id: None,
                created_at: now,
                updated_at: now,
            };
            if let Some(mode) = command.context_mode {
                document.context_mode = mode;
            }
            let requested = command.task_ids.as_deref().unwrap_or_default();
            let snapshot = Snapshot::for_task_refs(tx.db(), requested).await?;
            let wanted = resolve(&snapshot, requested)?;
            let scope = command.scope.unwrap_or_else(|| NoteScope::narrowest(&places(&snapshot, &wanted)));
            require_scope_owner(&tx, scope).await?;
            settle_scope(&snapshot, &mut document, scope, &wanted)?;
            document.validate()?;
            document.clone().into_active_model().insert(tx.db()).await?;
            link(&tx, document.id, &wanted).await?;
            response(&tx, &document).await
        })
    }

    pub async fn update(&self, id: Id, command: DocumentCommand) -> Result<DocumentView> {
        in_write!(&self.ctx, |tx| {
            let before = require(&tx, id).await?;
            let mut document = before.clone();
            document.title = command.title.clone().unwrap_or_default();
            document.kind = command.kind.clone().unwrap_or_default();
            document.body_markdown = body(&command);
            if let Some(mode) = command.context_mode {
                document.context_mode = mode;
            }
            let requested = command.task_ids.as_deref().unwrap_or_default();
            let snapshot = Snapshot::for_task_refs(tx.db(), requested).await?;
            let wanted = resolve(&snapshot, requested)?;
            let scope = command.scope.unwrap_or(before.scope());
            require_scope_owner(&tx, scope).await?;
            settle_scope(&snapshot, &mut document, scope, &wanted)?;
            link(&tx, document.id, &wanted).await?;
            let document = repo::update_if_changed(tx.db(), &before, document, self.ctx.now()).await?;
            response(&tx, &document).await
        })
    }

    pub async fn delete(&self, id: Id) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let document = require(&tx, id).await?;
            for link in repo::document::links_of_document(tx.db(), document.id).await? {
                detach(&tx, document.id, link.task_id).await?;
            }
            document::Entity::delete_by_id(document.id).exec(tx.db()).await?;
            Ok::<_, RekallError>(())
        })
    }
}

fn body(command: &DocumentCommand) -> String {
    command.body_markdown.clone().unwrap_or_default()
}

async fn require(tx: &Tx, id: Id) -> Result<document::Model> {
    repo::document::find_by_id(tx.db(), id).await?.ok_or_else(|| RekallError::not_found("Document", id))
}

/// The tasks a note is to be on, each required to exist. None is fine: its scope keeps it.
fn resolve(snapshot: &Snapshot, task_ids: &[Id]) -> Result<Vec<Id>> {
    let mut resolved = Vec::new();
    for id in task_ids {
        let task = snapshot.tasks.get(id).ok_or_else(|| RekallError::not_found("Task", *id))?;
        if !resolved.contains(&task.id) {
            resolved.push(task.id);
        }
    }
    Ok(resolved)
}

/// Each task's `(company, project)`, in the order given.
fn places(snapshot: &Snapshot, task_ids: &[Id]) -> Vec<(Id, Id)> {
    task_ids
        .iter()
        .filter_map(|id| snapshot.tasks.get(id))
        .filter_map(|task| snapshot.projects.get(&task.project_id).map(|project| (project.company_id, project.id)))
        .collect()
}

/// The company or project a scope names has to exist.
async fn require_scope_owner(tx: &Tx, scope: NoteScope) -> Result<()> {
    match scope {
        NoteScope::Global => Ok(()),
        NoteScope::Company { id } => repo::company::require(tx.db(), id).await.map(|_| ()),
        NoteScope::Project { id } => repo::project::require(tx.db(), id).await.map(|_| ()),
    }
}

/// Gives the note its scope once every task it is to be on sits inside it. A task outside is
/// refused rather than dropped, so narrowing a scope never unlinks silently.
fn settle_scope(snapshot: &Snapshot, document: &mut document::Model, scope: NoteScope, wanted: &[Id]) -> Result<()> {
    let outside: Vec<String> = wanted
        .iter()
        .filter_map(|id| snapshot.tasks.get(id))
        .filter_map(|task| snapshot.projects.get(&task.project_id).map(|project| (task, project)))
        .filter(|(_, project)| !scope.admits(project.company_id, project.id))
        .map(|(task, project)| format!("{}/{}", project.label, task.label))
        .collect();
    if !outside.is_empty() {
        return Err(RekallError::conflict(format!(
            "{} {} outside this note's scope. Take the note off {} or widen its scope first.",
            outside.join(", "),
            if outside.len() == 1 { "is" } else { "are" },
            if outside.len() == 1 { "it" } else { "them" }
        )));
    }
    document.set_scope(scope);
    Ok(())
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

/// The link goes, and the notes after it on that task move up one place, so positions stay dense.
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

async fn response(tx: &Tx, document: &document::Model) -> Result<DocumentView> {
    Ok(responses(tx, std::slice::from_ref(document)).await?.remove(0))
}

async fn responses(tx: &Tx, documents: &[document::Model]) -> Result<Vec<DocumentView>> {
    if documents.is_empty() {
        return Ok(Vec::new());
    }
    let mut links_per_document = Vec::with_capacity(documents.len());
    for document in documents {
        links_per_document.push(repo::document::links_of_document(tx.db(), document.id).await?);
    }
    let task_ids: Vec<Id> = links_per_document.iter().flatten().map(|link| link.task_id).collect();
    let snapshot = Snapshot::for_task_refs(tx.db(), &task_ids).await?;
    let mut out = Vec::with_capacity(documents.len());
    for (document, links) in documents.iter().zip(&links_per_document) {
        let mut tasks = Vec::new();
        for link in links {
            if let Some(task) = snapshot.tasks.get(&link.task_id) {
                tasks.push(snapshot.task_ref(task)?);
            }
        }
        out.push(DocumentView::of(document, tasks));
    }
    Ok(out)
}
