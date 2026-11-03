use rekall_console::adherence::StepVisitsView;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StepVisitsResponse {
    pub step_number: usize,
    pub title: String,
    pub visits: usize,
    pub max_visits: Option<i32>,
    pub budget_spent: bool,
}

impl From<StepVisitsView> for StepVisitsResponse {
    fn from(view: StepVisitsView) -> Self {
        Self { step_number: view.step_number, title: view.title, visits: view.visits, max_visits: view.max_visits, budget_spent: view.budget_spent }
    }
}
