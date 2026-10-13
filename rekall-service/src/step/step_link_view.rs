use rekall_common::Id;
use rekall_model::step_link;
use serde::Serialize;

/// A route leaving a step, as the console draws it and the context names it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StepLinkView {
    pub id: Id,
    pub from_step_id: Id,
    pub to_step_id: Id,
    pub outcome: String,
    pub position: i32,
}

impl StepLinkView {
    pub fn of(link: &step_link::Model) -> Self {
        Self {
            id: link.id,
            from_step_id: link.from_step_id,
            to_step_id: link.to_step_id,
            outcome: link.outcome.clone(),
            position: link.position,
        }
    }
}
