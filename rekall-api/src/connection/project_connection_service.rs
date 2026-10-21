//! Which of its company's connections a project uses, and for what. A project binds only a
//! connection of its own company.

use rekall_common::{Id, RekallError, Result};
use rekall_model::{external_connection, project, project_connection};
use rekall_repository::repository as repo;
use rekall_service::connection::ConnectionStreamEvent;
use rekall_service::{in_read, in_write, Ctx};

use super::connection_rules::target_of;
use super::{ProjectConnectionRequest, ProjectConnectionResponse};

#[derive(Clone)]
pub struct ProjectConnectionService {
    ctx: Ctx,
}

impl ProjectConnectionService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    pub async fn list(&self, project_id: Id) -> Result<Vec<ProjectConnectionResponse>> {
        in_read!(&self.ctx, |tx| {
            repo::project::require(tx.db(), project_id).await?;
            let rows = repo::project_connection::list_of_project(tx.db(), project_id).await?;
            Ok::<_, RekallError>(rows.iter().map(|(binding, connection)| ProjectConnectionResponse::of(binding, connection)).collect())
        })
    }

    /// Binds the connection, or sets the target of a binding the project already has.
    pub async fn bind(&self, project_id: Id, connection_id: Id, request: ProjectConnectionRequest) -> Result<ProjectConnectionResponse> {
        in_write!(&self.ctx, |tx| {
            let project = repo::project::require(tx.db(), project_id).await?;
            let connection = repo::external_connection::require(tx.db(), connection_id).await?;
            require_same_company(&project, &connection)?;
            let target = target_of(request.target.as_deref());
            let binding = match repo::project_connection::find(tx.db(), project_id, connection_id).await? {
                Some(before) => {
                    let after = project_connection::Model { target, ..before.clone() };
                    repo::update_if_changed(tx.db(), &before, after, self.ctx.now()).await?
                }
                None => repo::project_connection::bind(tx.db(), project_connection::Model::new(project_id, connection_id, target, self.ctx.now())).await?,
            };
            tx.publish(ConnectionStreamEvent { company_id: project.company_id });
            Ok::<_, RekallError>(ProjectConnectionResponse::of(&binding, &connection))
        })
    }

    pub async fn unbind(&self, project_id: Id, connection_id: Id) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let project = repo::project::require(tx.db(), project_id).await?;
            if !repo::project_connection::unbind(tx.db(), project_id, connection_id).await? {
                return Err(RekallError::not_found_msg("This project does not use that connection."));
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
