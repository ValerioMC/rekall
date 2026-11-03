use rekall_store::{step_visit, task_step};

/// How often a flow entered one step, against the budget the step allows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StepVisitsView {
    pub step_number: usize,
    pub title: String,
    pub visits: usize,
    pub max_visits: Option<i32>,
    pub budget_spent: bool,
}

impl StepVisitsView {
    pub fn of_flow(steps: &[task_step::Model], visits: &[step_visit::Model]) -> Vec<Self> {
        steps
            .iter()
            .enumerate()
            .filter_map(|(index, step)| {
                let entered = visits.iter().filter(|visit| visit.step_id == step.id).count();
                (entered > 0).then(|| Self {
                    step_number: index + 1,
                    title: step.title.clone(),
                    visits: entered,
                    max_visits: step.max_visits,
                    budget_spent: !step.admits_visit(entered),
                })
            })
            .collect()
    }
}
