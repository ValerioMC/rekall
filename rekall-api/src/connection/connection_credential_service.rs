//! The one path that stores, replaces or clears a connection's credential. Write-only: nothing here
//! returns a credential, and the row only learns when one was stored.

use std::sync::Arc;

use rekall_common::{Id, RekallError, Result};
use rekall_model::{external_connection, ConnectionKind};
use rekall_repository::repository as repo;
use rekall_service::connection::{ConnectionStreamEvent, JiraCredential, KubeAccess, Kubeconfig, Secret, SecretStore};
use rekall_service::{in_read, in_write, Ctx};

use super::credential_keeping::{blocking, forget, keep};
use super::{ConnectionCredentialRequest, ConnectionResponse};

#[derive(Clone)]
pub struct ConnectionCredentialService {
    ctx: Ctx,
    secrets: Arc<dyn SecretStore>,
}

/// What a request becomes once checked: the text the store keeps, and what the row learns from it.
struct PreparedCredential {
    stored: Secret,
    kube: Option<KubeAccess>,
}

impl ConnectionCredentialService {
    pub fn new(ctx: Ctx, secrets: Arc<dyn SecretStore>) -> Self {
        Self { ctx, secrets }
    }

    /// The store is written before the row, so a failed write leaves the row saying no credential is set.
    pub async fn set(&self, id: Id, request: ConnectionCredentialRequest) -> Result<ConnectionResponse> {
        let connection = self.connection(id).await?;
        let prepared = prepare(connection.kind, request).await?;
        keep(self.secrets.clone(), connection.secret_ref.clone(), prepared.stored).await?;
        in_write!(&self.ctx, |tx| {
            let before = repo::external_connection::require(tx.db(), id).await?;
            let mut after = before.clone();
            after.secret_set_at = Some(self.ctx.now());
            if let Some(kube) = &prepared.kube {
                after.base_url = Some(kube.server.clone());
                after.kube_context = Some(kube.context.clone());
            }
            let after = repo::update_if_changed(tx.db(), &before, after, self.ctx.now()).await?;
            tx.publish(ConnectionStreamEvent { company_id: after.company_id });
            let projects = repo::project_connection::list_of_connection(tx.db(), id).await?.len();
            Ok::<_, RekallError>(ConnectionResponse::of(&after, projects))
        })
    }

    /// The store is cleared first: a row that says "no credential" while the store still has one would lie.
    pub async fn clear(&self, id: Id) -> Result<ConnectionResponse> {
        let connection = self.connection(id).await?;
        forget(self.secrets.clone(), connection.secret_ref.clone()).await?;
        in_write!(&self.ctx, |tx| {
            let before = repo::external_connection::require(tx.db(), id).await?;
            let mut after = before.clone();
            after.secret_set_at = None;
            let after = repo::update_if_changed(tx.db(), &before, after, self.ctx.now()).await?;
            tx.publish(ConnectionStreamEvent { company_id: after.company_id });
            let projects = repo::project_connection::list_of_connection(tx.db(), id).await?.len();
            Ok::<_, RekallError>(ConnectionResponse::of(&after, projects))
        })
    }

    async fn connection(&self, id: Id) -> Result<external_connection::Model> {
        in_read!(&self.ctx, |tx| repo::external_connection::require(tx.db(), id).await)
    }
}

async fn prepare(kind: ConnectionKind, request: ConnectionCredentialRequest) -> Result<PreparedCredential> {
    match kind {
        ConnectionKind::Gitlab => gitlab_token(request).map(|token| PreparedCredential { stored: token, kube: None }),
        ConnectionKind::Kubernetes => kube_credential(request).await,
        ConnectionKind::Jira => jira_credential(request).map(|credential| PreparedCredential { stored: credential.to_stored(), kube: None }),
    }
}

/// An email makes it a Jira Cloud API token; without one, the token is a Data Center personal access token.
fn jira_credential(request: ConnectionCredentialRequest) -> Result<JiraCredential> {
    if request.kubeconfig.is_some() {
        return Err(RekallError::illegal("A Jira connection takes a token, not a kubeconfig."));
    }
    let token = request.token.ok_or_else(|| RekallError::illegal("A Jira connection needs an API token, or a personal access token on Data Center."))?;
    match request.email.as_deref().map(str::trim).filter(|email| !email.is_empty()) {
        Some(email) => JiraCredential::cloud(email, token),
        None => JiraCredential::data_center(token),
    }
}

fn gitlab_token(request: ConnectionCredentialRequest) -> Result<Secret> {
    if request.kubeconfig.is_some() {
        return Err(RekallError::illegal("A GitLab connection takes a token, not a kubeconfig."));
    }
    request.token.filter(|token| !token.is_blank()).ok_or_else(|| RekallError::illegal("A GitLab connection needs its personal access token."))
}

/// Parsing reads the files a kubeconfig names, so it runs off the async runtime.
async fn kube_credential(request: ConnectionCredentialRequest) -> Result<PreparedCredential> {
    if request.token.is_some() {
        return Err(RekallError::illegal("A Kubernetes connection takes a kubeconfig, not a token."));
    }
    let kubeconfig = request.kubeconfig.ok_or_else(|| RekallError::illegal("A Kubernetes connection needs its kubeconfig."))?;
    let context = request.context.filter(|context| !context.trim().is_empty()).ok_or_else(|| RekallError::illegal("Choose which context of the kubeconfig to keep."))?;
    blocking(move || {
        let kube = Kubeconfig::parse(&kubeconfig)?.select(&context)?;
        Ok(PreparedCredential { stored: kube.to_kubeconfig()?, kube: Some(kube) })
    })
    .await
}

#[cfg(test)]
#[path = "../../tests/unit/connection/connection_credential_service_tests.rs"]
mod tests;
