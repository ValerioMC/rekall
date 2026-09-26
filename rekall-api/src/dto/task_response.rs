use rekall_common::{Id, Instant};
use rekall_model::{TaskStatus, TaskStepState};
use serde::Serialize;

use super::TaskGraph;

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
