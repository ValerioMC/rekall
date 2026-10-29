//! A deployment graph as stored, with its steps in order and its routes, read back as the flow
//! it lays on a task.

use std::collections::HashMap;

use rekall_common::{Id, Result};
use rekall_model::{deployment_graph, graph_route, graph_step};
use rekall_service::flow::{FlowPlan, PlannedRoute, PlannedStep};
use rekall_service::step::VisitBudget;

use super::{DeploymentGraphView, GraphStepView};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredGraph {
    pub graph: deployment_graph::Model,
    pub steps: Vec<graph_step::Model>,
    pub routes: Vec<graph_route::Model>,
}

impl StoredGraph {
    /// The graph as a flow plan: routes point at positions, the first step is where it begins.
    pub fn plan(&self) -> Result<FlowPlan> {
        let positions: HashMap<Id, usize> = self.steps.iter().enumerate().map(|(at, step)| (step.id, at)).collect();
        Ok(FlowPlan::new(self.steps.iter().map(|step| self.planned(step, &positions)).collect::<Result<_>>()?))
    }

    /// Why the graph cannot run unattended, or `None` when it can. A graph with no routes runs as
    /// a plain checklist, so only a routed one has to pass the flow's rules.
    pub fn problem(&self) -> Result<Option<String>> {
        let plan = self.plan()?;
        if plan.route_count() == 0 {
            return Ok(None);
        }
        Ok(plan.check().err().map(|refusal| refusal.to_string()))
    }

    pub fn response(&self) -> Result<DeploymentGraphView> {
        let steps = self.steps.iter().map(|step| GraphStepView::of(step, &self.routes)).collect();
        Ok(DeploymentGraphView::of(&self.graph, steps, self.problem()?))
    }

    fn planned(&self, step: &graph_step::Model, positions: &HashMap<Id, usize>) -> Result<PlannedStep> {
        let routes = self
            .routes
            .iter()
            .filter(|route| route.from_step_id == step.id)
            .map(|route| PlannedRoute { outcome: route.outcome.clone(), to: route.to_step_id.and_then(|to| positions.get(&to).copied()) })
            .collect();
        Ok(PlannedStep {
            title: step.title.clone(),
            detail: step.body_markdown.clone(),
            routes,
            max_visits: step.max_visits.map(VisitBudget::new).transpose()?,
        })
    }
}

#[cfg(test)]
#[path = "../../tests/unit/graph/stored_graph_tests.rs"]
mod tests;
