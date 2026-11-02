//! What a person does to the workspace's connections: list, read, create, edit, share with
//! companies, delete. Deleting one also removes its credential from the OS credential store once
//! the row is gone.

use std::sync::Arc;

use rekall_kernel::{Id, RekallError, Result};
use rekall_store::external_connection;
use rekall_store::repository as repo;
use rekall_core::connection::{ConnectionStreamEvent, SecretStore};
use rekall_core::{in_read, in_write, Ctx, Tx};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};

use crate::connection::rule::connection_rules::{base_url_of, kind_of, label_of, proxy_url_of, root_path_of, title_of};
use crate::connection::support::connection_views::view_in;
use crate::connection::support::credential_keeping::forget_leftover;
use crate::connection::{ConnectionCommand, ConnectionCompanyView, ConnectionDetailView, ConnectionRemovedView, ConnectionView};

#[derive(Clone)]
pub struct ConnectionService {
    ctx: Ctx,
    secrets: Arc<dyn SecretStore>,
}

impl ConnectionService {
    pub fn new(ctx: Ctx, secrets: Arc<dyn SecretStore>) -> Self {
        Self { ctx, secrets }
    }

    pub async fn list_all(&self) -> Result<Vec<ConnectionView>> {
        in_read!(&self.ctx, |tx| {
            let connections = repo::external_connection::list_all(tx.db()).await?;
            views_of(&tx, &connections).await
        })
    }

    /// The connections shared with the company: the ones its projects can bind.
    pub async fn list_of_company(&self, company_id: Id) -> Result<Vec<ConnectionView>> {
        in_read!(&self.ctx, |tx| {
            repo::company::require(tx.db(), company_id).await?;
            let connections = self.list_in(&tx, company_id).await?;
            views_of(&tx, &connections).await
        })
    }

    /// The connections shared with the company, by label.
    pub async fn list_in(&self, tx: &Tx, company_id: Id) -> Result<Vec<external_connection::Model>> {
        repo::external_connection::list_of_company(tx.db(), company_id).await
    }

    pub async fn get(&self, id: Id) -> Result<ConnectionDetailView> {
        in_read!(&self.ctx, |tx| {
            let connection = repo::external_connection::require(tx.db(), id).await?;
            let companies = repo::connection_company::companies_of_connection(tx.db(), id).await?;
            let projects = repo::project_connection::list_of_connection(tx.db(), id).await?;
            Ok::<_, RekallError>(ConnectionDetailView::of(
                view_in(&tx, &connection).await?,
                companies.iter().map(ConnectionCompanyView::of).collect(),
                &projects,
            ))
        })
    }

    pub async fn create(&self, command: ConnectionCommand) -> Result<ConnectionView> {
        in_write!(&self.ctx, |tx| {
            let kind = kind_of(command.kind.as_deref())?;
            let title = title_of(command.title.as_deref())?;
            let label = label_of(command.label.as_deref(), &title)?;
            require_label_free(&tx, &label, None).await?;
            let base_url = base_url_of(kind, command.base_url.as_deref())?;
            let mut connection = external_connection::Model::new(label, title, kind, base_url, self.ctx.now());
            connection.proxy_url = proxy_url_of(command.proxy_url.as_deref());
            connection.root_path = root_path_of(command.root_path.as_deref());
            connection.validate()?;
            let company_ids = command.company_ids.unwrap_or_default();
            require_companies(&tx, &company_ids).await?;
            connection.clone().into_active_model().insert(tx.db()).await?;
            repo::connection_company::replace_companies(tx.db(), connection.id, &company_ids).await?;
            let view = view_in(&tx, &connection).await?;
            tx.publish(ConnectionStreamEvent { company_ids: view.company_ids.clone() });
            Ok::<_, RekallError>(view)
        })
    }

    pub async fn update(&self, id: Id, command: ConnectionCommand) -> Result<ConnectionView> {
        in_write!(&self.ctx, |tx| {
            let before = repo::external_connection::require(tx.db(), id).await?;
            let mut connection = before.clone();
            if command.kind.is_some() && kind_of(command.kind.as_deref())? != before.kind {
                return Err(RekallError::illegal("A connection keeps the kind it was created with. Create another one instead."));
            }
            if command.title.is_some() {
                connection.title = title_of(command.title.as_deref())?;
            }
            if command.label.is_some() {
                connection.label = label_of(command.label.as_deref(), &connection.title)?;
                require_label_free(&tx, &connection.label, Some(id)).await?;
            }
            if command.base_url.is_some() {
                connection.base_url = base_url_of(connection.kind, command.base_url.as_deref())?;
            }
            if command.proxy_url.is_some() {
                connection.proxy_url = proxy_url_of(command.proxy_url.as_deref());
            }
            if command.root_path.is_some() {
                connection.root_path = root_path_of(command.root_path.as_deref());
            }
            let changed = connection != before;
            let connection = repo::update_if_changed(tx.db(), &before, connection, self.ctx.now()).await?;
            let view = view_in(&tx, &connection).await?;
            if changed {
                tx.publish(ConnectionStreamEvent { company_ids: view.company_ids.clone() });
            }
            Ok::<_, RekallError>(view)
        })
    }

    /// The connection is shared with exactly these companies afterwards. A company is not removed
    /// while one of its projects still binds the connection: nothing is unbound silently.
    pub async fn set_companies(&self, id: Id, company_ids: Vec<Id>) -> Result<ConnectionView> {
        in_write!(&self.ctx, |tx| {
            let connection = repo::external_connection::require(tx.db(), id).await?;
            require_companies(&tx, &company_ids).await?;
            let before = repo::connection_company::company_ids_of_connection(tx.db(), id).await?;
            let removed: Vec<Id> = before.iter().copied().filter(|company_id| !company_ids.contains(company_id)).collect();
            require_no_binding_of(&tx, id, &removed).await?;
            repo::connection_company::replace_companies(tx.db(), id, &company_ids).await?;
            let view = view_in(&tx, &connection).await?;
            tx.publish(ConnectionStreamEvent { company_ids: affected_companies(&before, &view.company_ids) });
            Ok::<_, RekallError>(view)
        })
    }

    pub async fn delete(&self, id: Id) -> Result<ConnectionRemovedView> {
        let (secret_ref, unbound_projects) = in_write!(&self.ctx, |tx| {
            let connection = repo::external_connection::require(tx.db(), id).await?;
            let unbound = repo::project_connection::project_count(tx.db(), id).await?;
            let company_ids = repo::connection_company::company_ids_of_connection(tx.db(), id).await?;
            external_connection::Entity::delete_by_id(id).exec(tx.db()).await?;
            tx.publish(ConnectionStreamEvent { company_ids });
            Ok::<_, RekallError>((connection.secret_ref, unbound))
        })?;
        forget_leftover(self.secrets.clone(), secret_ref).await;
        Ok(ConnectionRemovedView { unbound_projects })
    }
}

async fn views_of(tx: &Tx, connections: &[external_connection::Model]) -> Result<Vec<ConnectionView>> {
    let mut views = Vec::with_capacity(connections.len());
    for connection in connections {
        views.push(view_in(tx, connection).await?);
    }
    Ok(views)
}

async fn require_label_free(tx: &Tx, label: &str, except: Option<Id>) -> Result<()> {
    if repo::external_connection::label_taken(tx.db(), label, except).await? {
        return Err(RekallError::conflict(format!("A connection is already labelled '{label}'.")));
    }
    Ok(())
}

async fn require_companies(tx: &Tx, company_ids: &[Id]) -> Result<()> {
    for company_id in company_ids {
        repo::company::require(tx.db(), *company_id).await?;
    }
    Ok(())
}

async fn require_no_binding_of(tx: &Tx, connection_id: Id, removed: &[Id]) -> Result<()> {
    if removed.is_empty() {
        return Ok(());
    }
    let mut blocking: Vec<Id> = repo::project_connection::list_of_connection(tx.db(), connection_id)
        .await?
        .into_iter()
        .filter(|(_, project)| removed.contains(&project.company_id))
        .map(|(_, project)| project.id)
        .collect();
    blocking.dedup();
    if blocking.is_empty() {
        return Ok(());
    }
    Err(RekallError::conflict(format!(
        "{} of the projects of the companies you removed still use this connection. Unbind them first.",
        blocking.len()
    )))
}

fn affected_companies(before: &[Id], after: &[Id]) -> Vec<Id> {
    let mut affected = before.to_vec();
    affected.extend(after.iter().copied().filter(|company_id| !before.contains(company_id)));
    affected
}

#[cfg(test)]
#[path = "../../../tests/unit/connection/service/connection_service_tests.rs"]
mod tests;
