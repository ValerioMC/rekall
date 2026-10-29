use rekall_kernel::{Id, Instant};
use rekall_console::catalog::TaskView;
use rekall_store::{TaskKind, TaskStatus, TaskStepState};
use serde::Serialize;

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
    pub graph_laid_at: Option<Instant>,
    pub release_skipped: bool,
    pub kind: TaskKind,
    pub epic_id: Option<Id>,
    pub anchor: String,
    pub updated_at: Instant,
}

impl From<TaskView> for TaskResponse {
    fn from(view: TaskView) -> Self {
        Self {
            id: view.id,
            label: view.label,
            title: view.title,
            status: view.status,
            description: view.description,
            project_id: view.project_id,
            project_label: view.project_label,
            project_title: view.project_title,
            company_name: view.company_name,
            project_repo_folder: view.project_repo_folder,
            document_count: view.document_count,
            step_count: view.step_count,
            steps_done: view.steps_done,
            draft_step_count: view.draft_step_count,
            has_wrapup: view.has_wrapup,
            review_state: view.review_state,
            review_active: view.review_active,
            claimed_at: view.claimed_at,
            accepted_at: view.accepted_at,
            review_note: view.review_note,
            tag_id: view.tag_id,
            tag_name: view.tag_name,
            tag_icon: view.tag_icon,
            tag_color: view.tag_color,
            graph_laid_at: view.graph_laid_at,
            release_skipped: view.release_skipped,
            kind: view.kind,
            epic_id: view.epic_id,
            anchor: view.anchor,
            updated_at: view.updated_at,
        }
    }
}
