//! Which connections a project uses, and for what. A project binds only a connection shared with
//! its own company, and may bind one several times, once per target.

use rekall_kernel::{Id, RekallError, Result};
use rekall_store::{external_connection, project, project_connection, ConnectionKind, Slug};
use rekall_store::repository as repo;
use rekall_core::connection::ConnectionStreamEvent;
use rekall_core::worktree::require_host_kept;
use rekall_core::{in_read, in_write, Ctx, Tx};

use crate::connection::rule::connection_rules::{first_free_label, target_of};
use crate::connection::rule::tracker_rules::tracker_location_of;
use crate::connection::{ProjectConnectionCommand, ProjectConnectionView};

#[derive(Clone)]
pub struct ProjectConnectionService {
    ctx: Ctx,
}

impl ProjectConnectionService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    pub async fn list(&self, project_id: Id) -> Result<Vec<ProjectConnectionView>> {
        in_read!(&self.ctx, |tx| {
            let rows = self.list_in(&tx, project_id).await?;
            Ok::<_, RekallError>(rows.iter().map(|(binding, connection)| ProjectConnectionView::of(binding, connection)).collect())
        })
    }

    /// The project's bindings with the connection each names, by label.
    pub async fn list_in(&self, tx: &Tx, project_id: Id) -> Result<Vec<(project_connection::Model, external_connection::Model)>> {
        repo::project::require(tx.db(), project_id).await?;
        repo::project_connection::list_of_project(tx.db(), project_id).await
    }

    /// One of the project's bindings, with the connection it binds.
    pub async fn binding(&self, project_id: Id, binding_id: Id) -> Result<(project_connection::Model, external_connection::Model)> {
        in_read!(&self.ctx, |tx| {
            let binding = repo::project_connection::require(tx.db(), project_id, binding_id).await?;
            let connection = repo::external_connection::require(tx.db(), binding.connection_id).await?;
            Ok::<_, RekallError>((binding, connection))
        })
    }

    pub async fn bind(&self, project_id: Id, connection_id: Id, command: ProjectConnectionCommand) -> Result<ProjectConnectionView> {
        in_write!(&self.ctx, |tx| {
            let (binding, connection) = self.bind_in(&mut tx, project_id, connection_id, command.label.as_deref(), target_of(command.target.as_deref())).await?;
            Ok::<_, RekallError>(ProjectConnectionView::of(&binding, &connection))
        })
    }

    /// Binds the connection under `label`, or under the connection's own label made free when none
    /// is typed. The same connection and target twice is refused: a second label would add nothing.
    pub async fn bind_in(
        &self,
        tx: &mut Tx,
        project_id: Id,
        connection_id: Id,
        label: Option<&str>,
        target: Option<String>,
    ) -> Result<(project_connection::Model, external_connection::Model)> {
        let project = repo::project::require(tx.db(), project_id).await?;
        let connection = repo::external_connection::require(tx.db(), connection_id).await?;
        require_associated(tx, &project, &connection).await?;
        let existing = repo::project_connection::list_of_project(tx.db(), project_id).await?;
        require_target_unbound(&existing, &connection, target.as_deref())?;
        let taken: Vec<String> = existing.iter().map(|(binding, _)| binding.label.clone()).collect();
        let label = match label.filter(|label| !label.trim().is_empty()) {
            Some(typed) => free_typed_label(typed, &taken)?,
            None => first_free_label(&connection.label, &taken),
        };
        let binding = repo::project_connection::bind(tx.db(), project_connection::Model::new(project_id, connection_id, label, target, self.ctx.now())).await?;
        tx.publish(ConnectionStreamEvent { company_ids: vec![project.company_id] });
        Ok((binding, connection))
    }

    /// The binding the project's tasks are written to, with its connection, when there is one.
    pub async fn task_manager(&self, project_id: Id) -> Result<Option<(project_connection::Model, external_connection::Model)>> {
        in_read!(&self.ctx, |tx| repo::project_connection::task_manager_of_project(tx.db(), project_id).await)
    }

    /// Sets the binding's target, and its label and whether it is the project's task manager when
    /// given. A project has one task manager: choosing a binding releases the one it had.
    pub async fn update(&self, project_id: Id, binding_id: Id, command: ProjectConnectionCommand) -> Result<ProjectConnectionView> {
        in_write!(&self.ctx, |tx| {
            let project = repo::project::require(tx.db(), project_id).await?;
            let before = repo::project_connection::require(tx.db(), project_id, binding_id).await?;
            let connection = repo::external_connection::require(tx.db(), before.connection_id).await?;
            let others: Vec<(project_connection::Model, external_connection::Model)> =
                repo::project_connection::list_of_project(tx.db(), project_id).await?.into_iter().filter(|(binding, _)| binding.id != binding_id).collect();
            let target = target_of(command.target.as_deref());
            require_target_unbound(&others, &connection, target.as_deref())?;
            let label = match command.label.as_deref() {
                Some(typed) => free_typed_label(typed, &others.iter().map(|(binding, _)| binding.label.clone()).collect::<Vec<_>>())?,
                None => before.label.clone(),
            };
            let task_manager = command.task_manager.unwrap_or(before.task_manager);
            if task_manager {
                tracker_location_of(connection.kind, target.as_deref())?;
                release_task_manager(&tx, &others, self.ctx.now()).await?;
            }
            if project.pull_request_flow {
                require_host_kept(task_manager_kind_after(task_manager.then_some(connection.kind), &others))?;
            }
            let after = project_connection::Model { label, target, task_manager, ..before.clone() };
            let binding = repo::update_if_changed(tx.db(), &before, after, self.ctx.now()).await?;
            tx.publish(ConnectionStreamEvent { company_ids: vec![project.company_id] });
            Ok::<_, RekallError>(ProjectConnectionView::of(&binding, &connection))
        })
    }

    pub async fn unbind(&self, project_id: Id, binding_id: Id) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let project = repo::project::require(tx.db(), project_id).await?;
            if project.pull_request_flow {
                let others: Vec<_> = repo::project_connection::list_of_project(tx.db(), project_id).await?.into_iter().filter(|(binding, _)| binding.id != binding_id).collect();
                require_host_kept(task_manager_kind_after(None, &others))?;
            }
            if !repo::project_connection::unbind(tx.db(), project_id, binding_id).await? {
                return Err(RekallError::not_found_msg("This project has no such connection."));
            }
            tx.publish(ConnectionStreamEvent { company_ids: vec![project.company_id] });
            Ok::<_, RekallError>(())
        })
    }
}

/// The kind of the project's task manager once a binding changes: the one it is set to, else the
/// manager among the other bindings.
fn task_manager_kind_after(chosen: Option<ConnectionKind>, others: &[(project_connection::Model, external_connection::Model)]) -> Option<ConnectionKind> {
    chosen.or_else(|| others.iter().find(|(binding, _)| binding.task_manager).map(|(_, connection)| connection.kind))
}

async fn release_task_manager(tx: &Tx, others: &[(project_connection::Model, external_connection::Model)], now: rekall_kernel::Instant) -> Result<()> {
    for (other, _) in others.iter().filter(|(binding, _)| binding.task_manager) {
        repo::update_if_changed(tx.db(), other, project_connection::Model { task_manager: false, ..other.clone() }, now).await?;
    }
    Ok(())
}

/// Another company's project has no business with a connection that was not shared with its company.
async fn require_associated(tx: &Tx, project: &project::Model, connection: &external_connection::Model) -> Result<()> {
    if !repo::connection_company::is_associated(tx.db(), connection.id, project.company_id).await? {
        return Err(RekallError::illegal("This connection is not shared with the project's company. Share it with that company first."));
    }
    Ok(())
}

fn require_target_unbound(bindings: &[(project_connection::Model, external_connection::Model)], connection: &external_connection::Model, target: Option<&str>) -> Result<()> {
    match bindings.iter().find(|(binding, _)| binding.connection_id == connection.id && binding.target.as_deref() == target) {
        Some((binding, _)) => Err(RekallError::conflict(format!("This project already uses {} for that, as '{}'.", connection.title, binding.label))),
        None => Ok(()),
    }
}

/// A label the person typed is theirs to choose, so a taken one is refused rather than renamed.
fn free_typed_label(typed: &str, taken: &[String]) -> Result<String> {
    let label = Slug::of(Some(typed))?;
    if taken.contains(&label) {
        return Err(RekallError::conflict(format!("This project already names a connection '{label}'.")));
    }
    Ok(label)
}
