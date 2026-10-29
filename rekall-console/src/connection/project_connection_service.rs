//! Which of its company's connections a project uses, and for what. A project binds only a
//! connection of its own company, and may bind one several times, once per target.

use rekall_common::{Id, RekallError, Result};
use rekall_store::{external_connection, project, project_connection, Slug};
use rekall_store::repository as repo;
use rekall_service::connection::ConnectionStreamEvent;
use rekall_service::{in_read, in_write, Ctx, Tx};

use super::connection_rules::{first_free_label, target_of};
use super::{ProjectConnectionCommand, ProjectConnectionView};

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
        require_same_company(&project, &connection)?;
        let existing = repo::project_connection::list_of_project(tx.db(), project_id).await?;
        require_target_unbound(&existing, &connection, target.as_deref())?;
        let taken: Vec<String> = existing.iter().map(|(binding, _)| binding.label.clone()).collect();
        let label = match label.filter(|label| !label.trim().is_empty()) {
            Some(typed) => free_typed_label(typed, &taken)?,
            None => first_free_label(&connection.label, &taken),
        };
        let binding = repo::project_connection::bind(tx.db(), project_connection::Model::new(project_id, connection_id, label, target, self.ctx.now())).await?;
        tx.publish(ConnectionStreamEvent { company_id: project.company_id });
        Ok((binding, connection))
    }

    /// Sets the binding's target, and its label when one is given.
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
            let after = project_connection::Model { label, target, ..before.clone() };
            let binding = repo::update_if_changed(tx.db(), &before, after, self.ctx.now()).await?;
            tx.publish(ConnectionStreamEvent { company_id: project.company_id });
            Ok::<_, RekallError>(ProjectConnectionView::of(&binding, &connection))
        })
    }

    pub async fn unbind(&self, project_id: Id, binding_id: Id) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let project = repo::project::require(tx.db(), project_id).await?;
            if !repo::project_connection::unbind(tx.db(), project_id, binding_id).await? {
                return Err(RekallError::not_found_msg("This project has no such connection."));
            }
            tx.publish(ConnectionStreamEvent { company_id: project.company_id });
            Ok::<_, RekallError>(())
        })
    }
}

/// A connection belongs to a company; another company's project has no business with its credential.
fn require_same_company(project: &project::Model, connection: &external_connection::Model) -> Result<()> {
    if project.company_id != connection.company_id {
        return Err(RekallError::illegal("A project can only use a connection of its own company."));
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
