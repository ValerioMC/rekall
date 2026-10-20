use rekall_common::{Id, Instant, Result};
use rekall_model::step_pass::StepPass;
use rekall_model::task_step;
use rekall_model::TaskStepState;
use serde::Serialize;

use super::{StepFlowRows, StepLinkView, StepVisitView};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskStepView {
    pub id: Id,
    pub task_id: Id,
    pub title: String,
    pub body_markdown: Option<String>,
    pub state: TaskStepState,
    pub done: bool,
    pub running_at: Option<Instant>,
    pub claimed_at: Option<Instant>,
    pub done_at: Option<Instant>,
    pub position: i32,
    /// Where the step was dragged to on the flow canvas, in canvas pixels.
    pub flow_x: Option<i32>,
    pub flow_y: Option<i32>,
    /// How many times the flow may enter the step; `None` sets no limit.
    pub max_visits: Option<i32>,
    /// The passes it was sent back from, oldest first; empty for a step on its first pass.
    pub passes: Vec<StepPass>,
    /// The routes leaving this step; a task whose steps carry any is a flow.
    pub links: Vec<StepLinkView>,
    /// Every entry of the flow into this step, oldest first; the open one is where it stands.
    pub visits: Vec<StepVisitView>,
    pub created_at: Instant,
    pub updated_at: Instant,
}

impl TaskStepView {
    pub fn of(step: &task_step::Model, flow: &StepFlowRows) -> Result<Self> {
        Ok(Self {
            id: step.id,
            task_id: step.task_id,
            title: step.title.clone(),
            body_markdown: step.body_markdown.clone(),
            state: step.state,
            done: step.is_done(),
            running_at: step.running_at,
            claimed_at: step.claimed_at,
            done_at: step.done_at,
            position: step.position,
            flow_x: step.flow_x,
            flow_y: step.flow_y,
            max_visits: step.max_visits,
            passes: step.passes()?,
            links: flow.links_from(step.id).map(StepLinkView::of).collect(),
            visits: flow.visits_of(step.id).map(StepVisitView::of).collect(),
            created_at: step.created_at,
            updated_at: step.updated_at,
        })
    }

    pub fn completed_at(&self) -> Option<Instant> {
        self.claimed_at.or(self.done_at)
    }

    /// How many times the flow has entered the step, against its budget when it has one: `visit 2 of 3`.
    pub fn visit_label(&self) -> String {
        match self.max_visits {
            Some(max) => format!("visit {} of {max}", self.visits.len()),
            None => format!("visit {}", self.visits.len()),
        }
    }

    /// Whether the flow stands on this step now.
    pub fn is_current(&self) -> bool {
        self.visits.iter().any(StepVisitView::is_open)
    }
}
