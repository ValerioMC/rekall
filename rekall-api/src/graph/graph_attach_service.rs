//! Lays a project's deployment graph on one of its tasks: each step of the graph becomes an open
//! step of the task, reaching the same connection, and each route a route between them. Console
//! only, like every write of the catalog.

use std::collections::HashMap;

use rekall_common::{Id, RekallError, Result};
use rekall_model::{deployment_graph, task};
use rekall_service::flow::{RouteTarget, StepLinkService};
use rekall_service::step::{TaskStepService, TaskStepView, VisitBudget};
use rekall_repository::repository as repo;
use rekall_service::{in_write, Ctx, Tx};

use super::{DeploymentGraphService, StoredGraph};

#[derive(Clone)]
pub struct GraphAttachService {
    ctx: Ctx,
    graphs: DeploymentGraphService,
    steps: TaskStepService,
    links: StepLinkService,
}

impl GraphAttachService {
    pub fn new(ctx: Ctx, graphs: DeploymentGraphService, steps: TaskStepService, links: StepLinkService) -> Self {
        Self { ctx, graphs, steps, links }
    }

    /// Only onto a task of the graph's project with no steps: the flow begins at the graph's first
    /// step, which a checklist already there would take the place of. Answers the task's steps.
    pub async fn attach(&self, task_id: Id, graph_id: Id) -> Result<Vec<TaskStepView>> {
        in_write!(&self.ctx, |tx| {
            let task = repo::task::require(tx.db(), task_id).await?;
            let stored = self.graphs.stored_in(&tx, graph_id).await?;
            require_same_project(&task, &stored.graph)?;
            require_no_steps(&self.steps.find_by_task_in(&mut tx, task.id).await?)?;
            require_runnable(&stored)?;
            let laid = self.lay_steps_in(&mut tx, task.id, &stored).await?;
            self.lay_routes_in(&tx, &stored, &laid).await?;
            self.steps.publish_in(&mut tx, task.id).await?;
            self.steps.find_by_task_in(&mut tx, task.id).await
        })
    }

    /// The task's new step for each of the graph's, by the graph step's id.
    async fn lay_steps_in(&self, tx: &mut Tx, task_id: Id, stored: &StoredGraph) -> Result<HashMap<Id, Id>> {
        let mut laid = HashMap::with_capacity(stored.steps.len());
        for step in &stored.steps {
            let added = self.steps.add_open_in(tx, task_id, Some(&step.title), step.body_markdown.as_deref()).await?;
            if let Some(visits) = step.max_visits {
                self.steps.limit_visits_in(tx, added.id, Some(VisitBudget::new(visits)?)).await?;
            }
            if step.connection_id.is_some() {
                self.steps.reach_connection_in(tx, added.id, step.connection_id).await?;
            }
            laid.insert(step.id, added.id);
        }
        Ok(laid)
    }

    async fn lay_routes_in(&self, tx: &Tx, stored: &StoredGraph, laid: &HashMap<Id, Id>) -> Result<()> {
        for route in &stored.routes {
            let from = *laid.get(&route.from_step_id).expect("every step of the graph was laid on the task");
            let target = route.to_step_id.map_or(RouteTarget::End, |to| {
                RouteTarget::Step(*laid.get(&to).expect("every step of the graph was laid on the task"))
            });
            self.links.add_in(tx, from, target, Some(&route.outcome)).await?;
        }
        Ok(())
    }
}

/// A graph names its project's connections, which another project's task may not reach.
fn require_same_project(task: &task::Model, graph: &deployment_graph::Model) -> Result<()> {
    if task.project_id == graph.project_id {
        return Ok(());
    }
    Err(RekallError::illegal(format!("'{}' belongs to another project. A task runs its own project's graphs.", graph.title)))
}

fn require_no_steps(existing: &[TaskStepView]) -> Result<()> {
    if existing.is_empty() {
        return Ok(());
    }
    Err(RekallError::conflict(format!(
        "This task already has {} steps. A graph is laid on a task with none, since its flow begins at the graph's first step.",
        existing.len()
    )))
}

fn require_runnable(stored: &StoredGraph) -> Result<()> {
    match stored.problem()? {
        None => Ok(()),
        Some(problem) => Err(RekallError::illegal(format!("'{}' cannot run yet: {problem}", stored.graph.title))),
    }
}
