use rekall_console::catalog::EpicConversionView;
use rekall_kernel::Id;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpicConversionResponse {
    pub task_id: Id,
    pub steps: usize,
    pub steps_done: usize,
    pub routes: usize,
    pub has_wrapup: bool,
    pub commits: usize,
    pub deviations: usize,
    pub waits_for: usize,
    pub needed_by: usize,
    pub skills: usize,
    pub has_pull_request: bool,
    pub epic_code: Option<String>,
    pub epic_title: Option<String>,
    pub queued: bool,
}

impl From<EpicConversionView> for EpicConversionResponse {
    fn from(view: EpicConversionView) -> Self {
        Self {
            task_id: view.task_id,
            steps: view.steps,
            steps_done: view.steps_done,
            routes: view.routes,
            has_wrapup: view.has_wrapup,
            commits: view.commits,
            deviations: view.deviations,
            waits_for: view.waits_for,
            needed_by: view.needed_by,
            skills: view.skills,
            has_pull_request: view.has_pull_request,
            epic_code: view.epic_code,
            epic_title: view.epic_title,
            queued: view.queued,
        }
    }
}
