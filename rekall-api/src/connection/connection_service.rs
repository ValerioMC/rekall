//! What a person does to a company's connections: list, read, create, edit, delete. Deleting one
//! also removes its credential from the OS credential store once the row is gone.

use std::sync::Arc;

use rekall_common::{Id, RekallError, Result};
use rekall_model::external_connection;
use rekall_repository::repository as repo;
use rekall_service::connection::{ConnectionStreamEvent, SecretStore};
use rekall_service::{in_read, in_write, Ctx, Tx};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};

use super::connection_rules::{base_url_of, kind_of, label_of, proxy_url_of, title_of};
use super::credential_keeping::forget_leftover;
use super::{ConnectionDetailResponse, ConnectionRemovedResponse, ConnectionRequest, ConnectionResponse};

#[derive(Clone)]
pub struct ConnectionService {
    ctx: Ctx,
    secrets: Arc<dyn SecretStore>,
}

impl ConnectionService {
    pub fn new(ctx: Ctx, secrets: Arc<dyn SecretStore>) -> Self {
        Self { ctx, secrets }
    }

    pub async fn list(&self, company_id: Id) -> Result<Vec<ConnectionResponse>> {
        in_read!(&self.ctx, |tx| {
            repo::company::require(tx.db(), company_id).await?;
            let connections = repo::external_connection::list_of_company(tx.db(), company_id).await?;
            let mut responses = Vec::with_capacity(connections.len());
            for connection in &connections {
                responses.push(ConnectionResponse::of(connection, project_count(&tx, connection.id).await?));
            }
            Ok::<_, RekallError>(responses)
        })
    }

    /// The company's connections, by label.
    pub async fn list_in(&self, tx: &Tx, company_id: Id) -> Result<Vec<external_connection::Model>> {
        repo::external_connection::list_of_company(tx.db(), company_id).await
    }

    pub async fn get(&self, id: Id) -> Result<ConnectionDetailResponse> {
        in_read!(&self.ctx, |tx| {
            let connection = repo::external_connection::require(tx.db(), id).await?;
            let projects = repo::project_connection::list_of_connection(tx.db(), id).await?;
            let project_count = repo::project_connection::project_count(tx.db(), id).await?;
            Ok::<_, RekallError>(ConnectionDetailResponse::of(&connection, &projects, project_count))
        })
    }

    pub async fn create(&self, company_id: Id, request: ConnectionRequest) -> Result<ConnectionResponse> {
        in_write!(&self.ctx, |tx| {
            repo::company::require(tx.db(), company_id).await?;
            let kind = kind_of(request.kind.as_deref())?;
            let title = title_of(request.title.as_deref())?;
            let label = label_of(request.label.as_deref(), &title)?;
            require_label_free(&tx, company_id, &label, None).await?;
            let base_url = base_url_of(kind, request.base_url.as_deref())?;
            let mut connection = external_connection::Model::new(company_id, label, title, kind, base_url, self.ctx.now());
            connection.proxy_url = proxy_url_of(request.proxy_url.as_deref());
            connection.validate()?;
            connection.clone().into_active_model().insert(tx.db()).await?;
            tx.publish(ConnectionStreamEvent { company_id });
            Ok::<_, RekallError>(ConnectionResponse::of(&connection, 0))
        })
    }

    pub async fn update(&self, id: Id, request: ConnectionRequest) -> Result<ConnectionResponse> {
        in_write!(&self.ctx, |tx| {
            let before = repo::external_connection::require(tx.db(), id).await?;
            let mut connection = before.clone();
            if request.kind.is_some() && kind_of(request.kind.as_deref())? != before.kind {
                return Err(RekallError::illegal("A connection keeps the kind it was created with. Create another one instead."));
            }
            if request.title.is_some() {
                connection.title = title_of(request.title.as_deref())?;
            }
            if request.label.is_some() {
                connection.label = label_of(request.label.as_deref(), &connection.title)?;
                require_label_free(&tx, connection.company_id, &connection.label, Some(id)).await?;
            }
            if request.base_url.is_some() {
                connection.base_url = base_url_of(connection.kind, request.base_url.as_deref())?;
            }
            if request.proxy_url.is_some() {
                connection.proxy_url = proxy_url_of(request.proxy_url.as_deref());
            }
            if connection != before {
                tx.publish(ConnectionStreamEvent { company_id: connection.company_id });
            }
            let connection = repo::update_if_changed(tx.db(), &before, connection, self.ctx.now()).await?;
            Ok::<_, RekallError>(ConnectionResponse::of(&connection, project_count(&tx, id).await?))
        })
    }

    pub async fn delete(&self, id: Id) -> Result<ConnectionRemovedResponse> {
        let (secret_ref, unbound_projects) = in_write!(&self.ctx, |tx| {
            let connection = repo::external_connection::require(tx.db(), id).await?;
            let unbound = project_count(&tx, id).await?;
            external_connection::Entity::delete_by_id(id).exec(tx.db()).await?;
            tx.publish(ConnectionStreamEvent { company_id: connection.company_id });
            Ok::<_, RekallError>((connection.secret_ref, unbound))
        })?;
        forget_leftover(self.secrets.clone(), secret_ref).await;
        Ok(ConnectionRemovedResponse { unbound_projects })
    }
}

async fn project_count(tx: &Tx, connection_id: Id) -> Result<usize> {
    repo::project_connection::project_count(tx.db(), connection_id).await
}

async fn require_label_free(tx: &Tx, company_id: Id, label: &str, except: Option<Id>) -> Result<()> {
    if repo::external_connection::label_taken(tx.db(), company_id, label, except).await? {
        return Err(RekallError::conflict(format!("This company already has a connection labelled '{label}'.")));
    }
    Ok(())
}
