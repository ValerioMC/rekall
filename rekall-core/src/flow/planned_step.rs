use super::PlannedRoute;
use crate::step::VisitBudget;

/// A draft a flow plan proposes, with the routes that leave it. No routes ends the flow there.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlannedStep {
    pub title: String,
    pub detail: Option<String>,
    pub routes: Vec<PlannedRoute>,
    /// How many times the flow may enter it; `None` sets no limit.
    pub max_visits: Option<VisitBudget>,
}
