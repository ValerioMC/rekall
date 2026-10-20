use super::PlannedRoute;

/// A draft a flow plan proposes, with the routes that leave it. No routes ends the flow there.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlannedStep {
    pub title: String,
    pub detail: Option<String>,
    pub routes: Vec<PlannedRoute>,
}
