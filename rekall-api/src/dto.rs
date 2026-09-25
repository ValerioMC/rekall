//! `ApiDtos`: what the console sends and what it gets back. Every field is written, nulls
//! included, because the console's schemas require the key to be there.

use rekall_common::{Id, Instant};
use rekall_model::{company, document, project, tag, task, task_step, wrapup};
use rekall_model::{DocumentContextMode, ProjectStatus, TaskStatus, TaskStepState};
use rekall_service::commit::RepositoryStatus;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyResponse {
    pub id: Id,
    pub name: String,
    pub description: Option<String>,
    pub project_count: usize,
    pub task_count: usize,
    pub updated_at: Instant,
}

impl CompanyResponse {
    pub fn of(company: &company::Model, project_count: usize, task_count: usize) -> Self {
        Self {
            id: company.id,
            name: company.name.clone(),
            description: company.description.clone(),
            project_count,
            task_count,
            updated_at: company.updated_at,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CompanyRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectResponse {
    pub id: Id,
    pub label: String,
    pub title: String,
    pub status: ProjectStatus,
    pub icon: String,
    pub description: Option<String>,
    pub blueprint_markdown: Option<String>,
    pub repo_folder: Option<String>,
    pub auto_commit: bool,
    pub company_id: Id,
    pub company_name: String,
    pub task_count: usize,
    pub anchor: String,
    pub updated_at: Instant,
}

impl ProjectResponse {
    pub fn of(project: &project::Model, company: &company::Model, task_count: usize) -> Self {
        Self {
            id: project.id,
            label: project.label.clone(),
            title: project.title.clone(),
            status: project.status,
            icon: project.icon.clone(),
            description: project.description.clone(),
            blueprint_markdown: project.blueprint_markdown.clone(),
            repo_folder: project.repo_folder.clone(),
            auto_commit: project.auto_commit,
            company_id: company.id,
            company_name: company.name.clone(),
            task_count,
            anchor: format!("project:{}", project.label),
            updated_at: project.updated_at,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ProjectRequest {
    pub label: Option<String>,
    pub title: Option<String>,
    pub status: Option<ProjectStatus>,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub blueprint_markdown: Option<String>,
    pub repo_folder: Option<String>,
    pub auto_commit: Option<bool>,
    pub company_id: Option<Id>,
}

/// What the project's folder is, git-wise, plus whether it auto-commits there.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRepositoryResponse {
    pub folder: Option<String>,
    pub exists: bool,
    pub repository: bool,
    pub branch: Option<String>,
    pub user_name: Option<String>,
    pub user_email: Option<String>,
    pub auto_commit: bool,
}

impl ProjectRepositoryResponse {
    pub fn of(status: RepositoryStatus, auto_commit: bool) -> Self {
        Self {
            folder: status.folder,
            exists: status.exists,
            repository: status.repository,
            branch: status.branch,
            user_name: status.user_name,
            user_email: status.user_email,
            auto_commit,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskResponse {
    pub id: Id,
    pub label: String,
    pub title: String,
    pub status: TaskStatus,
    pub description: Option<String>,
    pub project_id: Id,
    pub project_label: String,
    pub project_title: String,
    pub company_name: String,
    pub project_repo_folder: Option<String>,
    pub document_count: usize,
    pub step_count: usize,
    pub steps_done: usize,
    pub draft_step_count: usize,
    pub has_wrapup: bool,
    pub review_state: TaskStepState,
    pub review_active: bool,
    pub claimed_at: Option<Instant>,
    pub accepted_at: Option<Instant>,
    pub review_note: Option<String>,
    pub tag_id: Option<Id>,
    pub tag_name: Option<String>,
    pub tag_icon: Option<String>,
    pub tag_color: Option<String>,
    pub anchor: String,
    pub updated_at: Instant,
}

/// Everything a task row is built from.
pub struct TaskGraph<'a> {
    pub task: &'a task::Model,
    pub project: &'a project::Model,
    pub company: &'a company::Model,
    pub steps: Vec<&'a task_step::Model>,
    pub document_count: usize,
    pub wrapup: Option<&'a wrapup::Model>,
    pub tag: Option<&'a tag::Model>,
}

impl TaskResponse {
    pub fn of(graph: &TaskGraph<'_>) -> Self {
        let task = graph.task;
        let project = graph.project;
        Self {
            id: task.id,
            label: task.label.clone(),
            title: task.title.clone(),
            status: task.status,
            description: task.description.clone(),
            project_id: project.id,
            project_label: project.label.clone(),
            project_title: project.title.clone(),
            company_name: graph.company.name.clone(),
            project_repo_folder: project.repo_folder.clone(),
            document_count: graph.document_count,
            step_count: graph.steps.iter().filter(|s| !s.state.draft()).count(),
            steps_done: graph.steps.iter().filter(|s| s.is_done()).count(),
            draft_step_count: graph.steps.iter().filter(|s| s.state.draft()).count(),
            has_wrapup: graph.wrapup.is_some(),
            review_state: task.review_state,
            review_active: rekall_model::task::review_active(graph.steps.iter().map(|s| &s.state)),
            claimed_at: task.claimed_at,
            accepted_at: task.accepted_at,
            review_note: task.review_note.clone(),
            tag_id: graph.tag.map(|t| t.id),
            tag_name: graph.tag.map(|t| t.name.clone()),
            tag_icon: graph.tag.map(|t| t.icon.clone()),
            tag_color: graph.tag.map(|t| t.color.clone()),
            anchor: format!("project:{} task:{}", project.label, task.label),
            updated_at: task.updated_at,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TaskRequest {
    pub label: Option<String>,
    pub title: Option<String>,
    pub status: Option<TaskStatus>,
    pub description: Option<String>,
    pub project_id: Option<Id>,
    pub tag_id: Option<Id>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TagResponse {
    pub id: Id,
    pub name: String,
    pub icon: String,
    pub color: String,
    pub updated_at: Instant,
}

impl TagResponse {
    pub fn of(tag: &tag::Model) -> Self {
        Self { id: tag.id, name: tag.name.clone(), icon: tag.icon.clone(), color: tag.color.clone(), updated_at: tag.updated_at }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TagRequest {
    pub name: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
}

/// Accept (`DONE`) or send back (`OPEN`) a stepless task; other states are refused.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TaskReviewRequest {
    pub review_state: Option<TaskStepState>,
    pub note: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskRef {
    pub id: Id,
    pub label: String,
    pub title: String,
    pub project_label: String,
    pub project_title: String,
    pub company_name: String,
    pub anchor: String,
}

impl TaskRef {
    pub fn of(task: &task::Model, project: &project::Model, company: &company::Model) -> Self {
        Self {
            id: task.id,
            label: task.label.clone(),
            title: task.title.clone(),
            project_label: project.label.clone(),
            project_title: project.title.clone(),
            company_name: company.name.clone(),
            anchor: format!("project:{} task:{}", project.label, task.label),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentResponse {
    pub id: Id,
    pub title: String,
    pub kind: String,
    pub body_markdown: String,
    pub tasks: Vec<TaskRef>,
    pub context_mode: DocumentContextMode,
    pub anchor: String,
    pub updated_at: Instant,
}

impl DocumentResponse {
    pub fn of(document: &document::Model, tasks: Vec<TaskRef>) -> Self {
        Self {
            id: document.id,
            title: document.title.clone(),
            kind: document.kind.clone(),
            body_markdown: document.body_markdown.clone(),
            tasks,
            context_mode: document.context_mode,
            anchor: document.anchor(),
            updated_at: document.updated_at,
        }
    }
}

/// `contextMode` left out keeps what the note had, or `FULL` for a new one.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DocumentRequest {
    pub title: Option<String>,
    pub kind: Option<String>,
    pub body_markdown: Option<String>,
    pub task_ids: Option<Vec<Id>>,
    pub context_mode: Option<DocumentContextMode>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WrapupRequest {
    pub body_markdown: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TaskStepRequest {
    pub title: Option<String>,
    pub body_markdown: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TaskStepPatchRequest {
    pub title: Option<String>,
    pub body_markdown: Option<String>,
    pub done: Option<bool>,
    pub draft: Option<bool>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TaskStepMoveRequest {
    pub position: Option<i32>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TimeEntryEditRequest {
    pub started_at: Option<Instant>,
    pub stopped_at: Option<Instant>,
}

/// `stepId` is optional: omitted logs the commit against the task itself.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CommitReferenceRequest {
    pub step_id: Option<Id>,
}

/// A commit named by hand: the hash the picker chose or the person pasted; the step is optional.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PickedCommitReferenceRequest {
    pub step_id: Option<Id>,
    pub commit_hash: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitReferenceDiffResponse {
    pub diff: Option<String>,
}

/// An absent key reads as off, not as a failed request.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CommitReferenceContextRequest {
    pub in_context: Option<bool>,
}

/// One folder as the path picker shows it. Every path in it is absolute and already resolved, so
/// the page never joins, splits or normalises a path itself and never has to know the platform's
/// separator. `readable` is false for a folder that exists but refused to be listed (a macOS
/// privacy-protected one, say): the picker still stands in it and says why it is empty, rather
/// than failing the whole request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryListingResponse {
    pub path: String,
    pub parent: Option<String>,
    pub home: String,
    pub segments: Vec<PathSegmentResponse>,
    pub entries: Vec<DirectoryEntryResponse>,
    pub readable: bool,
    pub truncated: bool,
}

/// One crumb of a listing's breadcrumb, from the filesystem root down to the folder itself.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PathSegmentResponse {
    pub name: String,
    pub path: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryEntryResponse {
    pub name: String,
    pub path: String,
    pub directory: bool,
    pub hidden: bool,
}
