//! A graph as a request writes it, made into a flow plan: each step's routes resolved from the
//! caller's keys to positions, outcomes as slugs, and the shape every flow has to hold checked.

use std::collections::HashMap;

use rekall_common::{jstr, Id, RekallError, Result};
use rekall_store::step_link;
use rekall_service::flow::{FlowPlan, PlannedRoute, PlannedStep};
use rekall_service::step::{FlowPlace, VisitBudget};

use super::{GraphRouteCommand, GraphStepCommand};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphDraft {
    pub plan: FlowPlan,
    /// The connection each step of `plan` reaches, by position.
    pub connections: Vec<Option<Id>>,
    /// Where each step of `plan` was dragged on the canvas, by position.
    pub places: Vec<Option<FlowPlace>>,
}

impl GraphDraft {
    pub fn of(steps: &[GraphStepCommand]) -> Result<Self> {
        let positions = positions_by_key(steps)?;
        let mut planned = Vec::with_capacity(steps.len());
        for step in steps {
            planned.push(PlannedStep {
                title: step.title.as_deref().map(jstr::strip).unwrap_or_default().to_string(),
                detail: step.detail.as_deref().filter(|detail| !jstr::is_blank(detail)).map(str::to_string),
                routes: routes_of(step, &positions)?,
                max_visits: step.max_visits.map(VisitBudget::new).transpose()?,
            });
        }
        let plan = FlowPlan::new(planned);
        plan.check_shape()?;
        Ok(Self {
            plan,
            connections: steps.iter().map(|step| step.connection_id).collect(),
            places: steps.iter().map(place_of).collect::<Result<_>>()?,
        })
    }

    /// Each step with the connection it reaches, for the steps that reach one.
    pub fn reaching(&self) -> impl Iterator<Item = (&PlannedStep, Id)> {
        self.plan.steps.iter().zip(&self.connections).filter_map(|(step, connection)| connection.map(|id| (step, id)))
    }
}

/// Where each key sits in the request. A key names one step, so a blank or repeated one is refused.
fn positions_by_key(steps: &[GraphStepCommand]) -> Result<HashMap<&str, usize>> {
    let mut positions = HashMap::with_capacity(steps.len());
    for (at, step) in steps.iter().enumerate() {
        let key = step.key.as_deref().map(jstr::strip).filter(|key| !key.is_empty()).ok_or_else(|| {
            RekallError::illegal(format!("Step {} has no key. Every step needs one, for the routes to name it.", at + 1))
        })?;
        if positions.insert(key, at).is_some() {
            return Err(RekallError::illegal(format!("The key '{key}' names two steps. A key names one.")));
        }
    }
    Ok(positions)
}

/// Where the step sits on the canvas: both coordinates or neither, which leaves it to the layout.
fn place_of(step: &GraphStepCommand) -> Result<Option<FlowPlace>> {
    match (step.flow_x, step.flow_y) {
        (None, None) => Ok(None),
        (Some(x), Some(y)) => FlowPlace::new(x, y).map(Some),
        _ => Err(RekallError::illegal(format!(
            "'{}' has only one coordinate. A step's place takes both, or neither for the automatic layout.",
            step.title.as_deref().unwrap_or_default()
        ))),
    }
}

fn routes_of(step: &GraphStepCommand, positions: &HashMap<&str, usize>) -> Result<Vec<PlannedRoute>> {
    step.routes.as_deref().unwrap_or_default().iter().map(|route| route_of(route, positions)).collect()
}

/// A route by its outcome as a slug, and its target's position; no key leads to the end.
fn route_of(route: &GraphRouteCommand, positions: &HashMap<&str, usize>) -> Result<PlannedRoute> {
    let outcome = step_link::normalise_outcome(route.outcome.as_deref())?;
    let to = match route.to_key.as_deref().map(jstr::strip) {
        None => None,
        Some(key) => Some(*positions.get(key).ok_or_else(|| {
            RekallError::illegal(format!("The route `{outcome}` leads to '{key}', which is not a step of this graph."))
        })?),
    };
    Ok(PlannedRoute { outcome, to })
}

#[cfg(test)]
#[path = "../../tests/unit/graph/graph_draft_tests.rs"]
mod tests;
