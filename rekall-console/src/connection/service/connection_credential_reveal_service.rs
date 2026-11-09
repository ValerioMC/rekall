//! The one path that reads a stored credential back, for the console's reveal and copy buttons.
//! Only the console reaches it: `rekall-mcp` and a session's helpers never do.

use std::sync::Arc;

use rekall_kernel::{Id, RekallError, Result};
use rekall_store::{external_connection, ConnectionKind};
use rekall_store::repository as repo;
use rekall_core::connection::{JiraCredential, Secret, SecretStore};
use rekall_core::{in_read, Ctx};

use crate::connection::support::credential_keeping::recall;
use crate::connection::RevealedCredentialView;

#[derive(Clone)]
pub struct ConnectionCredentialRevealService {
    ctx: Ctx,
    secrets: Arc<dyn SecretStore>,
}

impl ConnectionCredentialRevealService {
    pub fn new(ctx: Ctx, secrets: Arc<dyn SecretStore>) -> Self {
        Self { ctx, secrets }
    }

    pub async fn reveal(&self, id: Id) -> Result<RevealedCredentialView> {
        let connection = in_read!(&self.ctx, |tx| repo::external_connection::require(tx.db(), id).await)?;
        if connection.secret_set_at.is_none() {
            return Err(RekallError::not_found_msg(format!("{} has no credential stored.", connection.title)));
        }
        let stored = recall(self.secrets.clone(), connection.secret_ref.clone()).await?;
        revealed(&connection, stored)
    }
}

fn revealed(connection: &external_connection::Model, stored: Secret) -> Result<RevealedCredentialView> {
    match connection.kind {
        ConnectionKind::Gitlab | ConnectionKind::Github | ConnectionKind::Grafana => Ok(RevealedCredentialView { token: Some(stored), kubeconfig: None, email: None }),
        ConnectionKind::Kubernetes => Ok(RevealedCredentialView { token: None, kubeconfig: Some(stored), email: None }),
        ConnectionKind::Jira => {
            let jira = JiraCredential::from_stored(&stored)?;
            Ok(RevealedCredentialView { token: Some(Secret::new(jira.token().expose())), kubeconfig: None, email: jira.email().map(str::to_string) })
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/connection/service/connection_credential_reveal_service_tests.rs"]
mod tests;
