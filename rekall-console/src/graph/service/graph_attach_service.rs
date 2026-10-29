//! Lays a project's deployment graph on one of its tasks, after the task's own work: each step of
//! the graph becomes an open step of the task, reaching the same connection, each route a route
//! between them, and the work leads into the graph's first step. By hand, or on its own with the
//! project's release graph when the console runs the task. Console only, like every write of the
//! catalog.

use std::collections::HashMap;

use rekall_kernel::{jstr, Id, RekallError, Result};
use rekall_store::{deployment_graph, task};
use rekall_core::flow::{RouteTarget, StepLinkService, StepVisitService};
use rekall_core::step::{TaskStepService, TaskStepView, VisitBudget};
use rekall_store::repository as repo;
use rekall_core::{in_write, Ctx, Tx};

use crate::graph::{DeploymentGraphService, ReleaseOnRunView, StoredGraph};
use crate::catalog::TaskService;

/// The step laid ahead of the graph on a task that has only its description, so the flow has a
/// place where the work is done before it is released.
pub const IMPLEMENTATION_TITLE: &str = "Implement the task";

const IMPLEMENTATION_DETAIL: &str = "Build what the task's description asks for: on this step the description is the brief, \
and the steps after this one release the work.\n\nClaim `done` once the work is complete. On a project that commits on \
claim, this claim is the commit the push sends.";

/// The outcome by which a step of the task's own work leads on to the next one.
const WORK_DONE: &str = "done";

#[derive(Clone)]
pub struct GraphAttachService {
    ctx: Ctx,
    graphs: DeploymentGraphService,
    tasks: TaskService,
    steps: TaskStepService,
    links: StepLinkService,
    visits: StepVisitService,
}

impl GraphAttachService {
    pub fn new(
        ctx: Ctx,
        graphs: DeploymentGraphService,
        tasks: TaskService,
        steps: TaskStepService,
        links: StepLinkService,
        visits: StepVisitService,
    ) -> Self {
        Self { ctx, graphs, tasks, steps, links, visits }
    }

    /// Lays the graph after the task's steps, or after a generated implementation step on a task
    /// with none, and leads the work into the graph's first step. Answers the task's steps.
    pub async fn attach(&self, task_id: Id, graph_id: Id) -> Result<Vec<TaskStepView>> {
        in_write!(&self.ctx, |tx| {
            let task = repo::task::require(tx.db(), task_id).await?;
            let stored = self.graphs.stored_in(&tx, graph_id).await?;
            self.lay_in(&mut tx, &task, &stored).await
        })
    }

    /// The project's release graph laid as `attach` would, unless the task already has a graph,
    /// is left unreleased or the project chose none: then nothing changes.
    pub async fn release_on_run(&self, task_id: Id) -> Result<ReleaseOnRunView> {
        in_write!(&self.ctx, |tx| {
            let task = repo::task::require(tx.db(), task_id).await?;
            let release = match (task.graph_laid_at, task.release_skipped) {
                (None, false) => self.graphs.release_default_in(&tx, task.project_id).await?,
                _ => None,
            };
            let Some(stored) = release else {
                let steps = self.steps.find_by_task_in(&mut tx, task.id).await?;
                return Ok(ReleaseOnRunView { laid: None, steps });
            };
            let steps = self.lay_in(&mut tx, &task, &stored).await?;
            Ok(ReleaseOnRunView { laid: Some(stored.graph.title.clone()), steps })
        })
    }

    async fn lay_in(&self, tx: &mut Tx, task: &task::Model, stored: &StoredGraph) -> Result<Vec<TaskStepView>> {
        task.require_workable("is never laid on a deployment graph")?;
        require_same_project(task, &stored.graph)?;
        require_runnable(stored)?;
        let mut work = self.steps.find_by_task_in(tx, task.id).await?;
        require_no_drafts(&work)?;
        require_free_titles(&work, stored)?;
        if work.is_empty() {
            work.push(self.steps.add_open_in(tx, task.id, Some(IMPLEMENTATION_TITLE), Some(IMPLEMENTATION_DETAIL)).await?);
        }
        let laid = self.lay_steps_in(tx, task.id, stored).await?;
        self.lay_routes_in(tx, stored, &laid).await?;
        let entry = *laid.get(&stored.steps[0].id).expect("a graph that can run has a first step, and every step was laid");
        self.lead_work_into_in(tx, &work, entry).await?;
        self.stand_on_first_unfinished_in(tx, task.id).await?;
        self.tasks.mark_graph_laid_in(tx, task.id).await?;
        self.steps.publish_in(tx, task.id).await?;
        self.steps.find_by_task_in(tx, task.id).await
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

    /// Wherever the work would end, it goes on into the graph instead. A plain checklist is first
    /// chained in its order, since on a flow nothing follows a step by its place in the list.
    async fn lead_work_into_in(&self, tx: &Tx, work: &[TaskStepView], entry: Id) -> Result<()> {
        if work.iter().all(|step| step.links.is_empty()) {
            return self.chain_work_in(tx, work, entry).await;
        }
        for step in work {
            if step.links.is_empty() {
                self.links.add_in(tx, step.id, RouteTarget::Step(entry), Some(WORK_DONE)).await?;
            }
            for ending in step.links.iter().filter(|link| link.to_step_id.is_none()) {
                self.links.retarget_in(tx, ending.id, RouteTarget::Step(entry)).await?;
            }
        }
        Ok(())
    }

    async fn chain_work_in(&self, tx: &Tx, work: &[TaskStepView], entry: Id) -> Result<()> {
        let targets = work.iter().skip(1).map(|step| step.id).chain(std::iter::once(entry));
        for (step, next) in work.iter().zip(targets) {
            self.links.add_in(tx, step.id, RouteTarget::Step(next), Some(WORK_DONE)).await?;
        }
        Ok(())
    }

    /// A flow begins at its first step, so when that one is already finished and nothing is
    /// current, the flow is set on the first unfinished step: the work there, or the release.
    async fn stand_on_first_unfinished_in(&self, tx: &mut Tx, task_id: Id) -> Result<()> {
        let steps = self.steps.find_by_task_in(tx, task_id).await?;
        if steps.iter().any(TaskStepView::is_current) || !steps.first().is_some_and(|first| first.state.complete()) {
            return Ok(());
        }
        let Some(unfinished) = steps.iter().find(|step| !step.state.complete()) else {
            return Ok(());
        };
        let step = repo::task_step::require(tx.db(), unfinished.id).await?;
        self.visits.enter_in(tx, &step).await?;
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

/// A draft is outside the flow and stays there once promoted, since no route reaches it.
fn require_no_drafts(existing: &[TaskStepView]) -> Result<()> {
    let drafts = existing.iter().filter(|step| step.state.draft()).count();
    if drafts == 0 {
        return Ok(());
    }
    Err(RekallError::conflict(format!(
        "This task has {drafts} draft steps. Promote or delete them before laying a graph, since no route of the flow would reach them."
    )))
}

/// A step is claimed by its title, so two steps under one would be ambiguous; it is also how a
/// graph laid twice is refused.
fn require_free_titles(existing: &[TaskStepView], stored: &StoredGraph) -> Result<()> {
    let taken: Vec<&str> = if existing.is_empty() {
        vec![IMPLEMENTATION_TITLE]
    } else {
        existing.iter().map(|step| step.title.as_str()).collect()
    };
    let Some(clash) = stored.steps.iter().find(|step| taken.iter().any(|title| jstr::equals_ignore_case(title, &step.title))) else {
        return Ok(());
    };
    Err(RekallError::conflict(format!(
        "This task already has a step named '{}'. A graph is laid once, and its steps' titles must be new on the task.",
        clash.title
    )))
}

fn require_runnable(stored: &StoredGraph) -> Result<()> {
    match stored.problem()? {
        None => Ok(()),
        Some(problem) => Err(RekallError::illegal(format!("'{}' cannot run yet: {problem}", stored.graph.title))),
    }
}
