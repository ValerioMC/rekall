//! What a person does to a project's deployment graphs: list them, write one whole, rewrite it,
//! delete it. Console-only: a session meets a graph only as the steps it lays on a task.

use rekall_kernel::{jstr, Id, RekallError, Result};
use rekall_store::{deployment_graph, graph_route, graph_step};
use rekall_store::repository as repo;
use rekall_core::{in_read, in_write, Ctx, Tx};
use sea_orm::{ActiveModelTrait, IntoActiveModel};

use super::{DeploymentGraphCommand, DeploymentGraphView, GraphDraft, StoredGraph};

#[derive(Clone)]
pub struct DeploymentGraphService {
    ctx: Ctx,
}

impl DeploymentGraphService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    pub async fn list(&self, project_id: Id) -> Result<Vec<DeploymentGraphView>> {
        in_read!(&self.ctx, |tx| {
            repo::project::require(tx.db(), project_id).await?;
            let mut graphs = Vec::new();
            for graph in repo::deployment_graph::list_of_project(tx.db(), project_id).await? {
                graphs.push(self.stored_of(&tx, graph).await?.response()?);
            }
            Ok::<_, RekallError>(graphs)
        })
    }

    pub async fn create(&self, project_id: Id, command: DeploymentGraphCommand) -> Result<DeploymentGraphView> {
        let title = title_of(command.title.as_deref())?;
        let draft = GraphDraft::of(command.steps.as_deref().unwrap_or_default())?;
        in_write!(&self.ctx, |tx| {
            repo::project::require(tx.db(), project_id).await?;
            require_title_free(&tx, project_id, &title, None).await?;
            require_connections_bound(&tx, project_id, &draft).await?;
            let graph = deployment_graph::Model::new(project_id, title, description_of(command.description.as_deref()), self.ctx.now());
            graph.validate()?;
            graph.clone().into_active_model().insert(tx.db()).await?;
            self.write_steps_in(&tx, &graph, &draft).await?;
            self.stored_of(&tx, graph).await?.response()
        })
    }

    /// Replaces the graph's title, description and every step and route with the request's.
    pub async fn replace(&self, id: Id, command: DeploymentGraphCommand) -> Result<DeploymentGraphView> {
        let title = title_of(command.title.as_deref())?;
        let draft = GraphDraft::of(command.steps.as_deref().unwrap_or_default())?;
        in_write!(&self.ctx, |tx| {
            let before = repo::deployment_graph::require(tx.db(), id).await?;
            require_title_free(&tx, before.project_id, &title, Some(id)).await?;
            require_connections_bound(&tx, before.project_id, &draft).await?;
            let graph = deployment_graph::Model {
                title,
                description: description_of(command.description.as_deref()),
                updated_at: self.ctx.now(),
                ..before
            };
            graph.validate()?;
            graph.clone().into_active_model().reset_all().update(tx.db()).await?;
            repo::graph_step::delete_of_graph(tx.db(), id).await?;
            self.write_steps_in(&tx, &graph, &draft).await?;
            self.stored_of(&tx, graph).await?.response()
        })
    }

    pub async fn delete(&self, id: Id) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            repo::deployment_graph::require(tx.db(), id).await?;
            repo::deployment_graph::delete(tx.db(), id).await
        })
    }

    /// Makes `graph_id` the one graph the project releases every task it runs with, or none.
    pub async fn choose_release(&self, project_id: Id, graph_id: Option<Id>) -> Result<Vec<DeploymentGraphView>> {
        in_write!(&self.ctx, |tx| {
            repo::project::require(tx.db(), project_id).await?;
            let graphs = repo::deployment_graph::list_of_project(tx.db(), project_id).await?;
            if let Some(chosen) = graph_id {
                require_of_project(&tx, chosen, project_id).await?;
            }
            let mut answered = Vec::with_capacity(graphs.len());
            for before in graphs {
                let graph = deployment_graph::Model { release_default: Some(before.id) == graph_id, ..before.clone() };
                let graph = repo::update_if_changed(tx.db(), &before, graph, self.ctx.now()).await?;
                answered.push(self.stored_of(&tx, graph).await?.response()?);
            }
            Ok::<_, RekallError>(answered)
        })
    }

    /// The project's release graph with its steps and routes, inside a transaction the caller holds.
    pub async fn release_default_in(&self, tx: &Tx, project_id: Id) -> Result<Option<StoredGraph>> {
        match repo::deployment_graph::release_default_of_project(tx.db(), project_id).await? {
            Some(graph) => Ok(Some(self.stored_of(tx, graph).await?)),
            None => Ok(None),
        }
    }

    /// The graph with its steps and routes, inside a transaction the caller holds.
    pub async fn stored_in(&self, tx: &Tx, id: Id) -> Result<StoredGraph> {
        let graph = repo::deployment_graph::require(tx.db(), id).await?;
        self.stored_of(tx, graph).await
    }

    async fn stored_of(&self, tx: &Tx, graph: deployment_graph::Model) -> Result<StoredGraph> {
        let steps = repo::graph_step::list_of_graph(tx.db(), graph.id).await?;
        let routes = repo::graph_route::list_of_graph(tx.db(), graph.id).await?;
        Ok(StoredGraph { graph, steps, routes })
    }

    async fn write_steps_in(&self, tx: &Tx, graph: &deployment_graph::Model, draft: &GraphDraft) -> Result<()> {
        let now = self.ctx.now();
        let mut ids = Vec::with_capacity(draft.plan.steps.len());
        for (position, ((planned, connection_id), place)) in draft.plan.steps.iter().zip(&draft.connections).zip(&draft.places).enumerate() {
            let step = graph_step::Model {
                id: Id::random(),
                graph_id: graph.id,
                position: position as i32,
                title: planned.title.clone(),
                body_markdown: planned.detail.clone(),
                connection_id: *connection_id,
                max_visits: planned.max_visits.map(|budget| budget.visits()),
                flow_x: place.map(|at| at.x),
                flow_y: place.map(|at| at.y),
                created_at: now,
            };
            step.validate()?;
            ids.push(step.clone().into_active_model().insert(tx.db()).await?.id);
        }
        for (from, planned) in ids.iter().zip(&draft.plan.steps) {
            for (rank, route) in planned.routes.iter().enumerate() {
                let to_step_id = route.to.map(|to| *ids.get(to).expect("GraphDraft::of keeps every route inside the graph"));
                let row = graph_route::Model { id: Id::random(), graph_id: graph.id, from_step_id: *from, to_step_id, outcome: route.outcome.clone(), position: rank as i32 };
                row.validate()?;
                row.into_active_model().insert(tx.db()).await?;
            }
        }
        Ok(())
    }
}

async fn require_of_project(tx: &Tx, graph_id: Id, project_id: Id) -> Result<()> {
    let graph = repo::deployment_graph::require(tx.db(), graph_id).await?;
    if graph.project_id == project_id {
        return Ok(());
    }
    Err(RekallError::illegal(format!("'{}' belongs to another project. A project releases with its own graphs.", graph.title)))
}

fn title_of(raw: Option<&str>) -> Result<String> {
    raw.map(jstr::strip).filter(|title| !title.is_empty()).map(str::to_string).ok_or_else(|| RekallError::illegal("A deployment graph needs a title."))
}

fn description_of(raw: Option<&str>) -> String {
    raw.map(jstr::strip).unwrap_or_default().to_string()
}

async fn require_title_free(tx: &Tx, project_id: Id, title: &str, except: Option<Id>) -> Result<()> {
    let graphs = repo::deployment_graph::list_of_project(tx.db(), project_id).await?;
    match graphs.iter().find(|graph| Some(graph.id) != except && jstr::equals_ignore_case(&graph.title, title)) {
        Some(existing) => Err(RekallError::conflict(format!("This project already has a deployment graph named '{}'.", existing.title))),
        None => Ok(()),
    }
}

/// A step reaches only what its project binds: the binding is what lists the connection in a
/// session's context, and another company's connection is never the project's to use.
async fn require_connections_bound(tx: &Tx, project_id: Id, draft: &GraphDraft) -> Result<()> {
    for (step, connection_id) in draft.reaching() {
        if !repo::project_connection::is_bound(tx.db(), project_id, connection_id).await? {
            return Err(RekallError::illegal(format!(
                "'{}' reaches a connection this project does not use. Bind the connection to the project first.",
                step.title
            )));
        }
    }
    Ok(())
}
