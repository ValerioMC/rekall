use std::sync::Arc;

use rekall_common::{Id, InternalFailure, RekallError, Result};
use rekall_model::{external_connection, ConnectionKind};
use rekall_repository::repository as repo;
use rekall_service::connection::{JiraCredential, JiraSite, KubeAccess, Secret, SecretStore};
use rekall_service::{in_read, Ctx};

use super::{ConnectionCheck, ConnectionProbe, ProbeOutcome};

/// Reads a connection's credential and asks its server with it. The answer is shown, not stored.
#[derive(Clone)]
pub struct ConnectionCheckService {
    ctx: Ctx,
    secrets: Arc<dyn SecretStore>,
    probe: Arc<dyn ConnectionProbe>,
}

impl ConnectionCheckService {
    pub fn new(ctx: Ctx, secrets: Arc<dyn SecretStore>, probe: Arc<dyn ConnectionProbe>) -> Self {
        Self { ctx, secrets, probe }
    }

    pub async fn check(&self, id: Id) -> Result<ConnectionCheck> {
        let connection = in_read!(&self.ctx, |tx| repo::external_connection::require(tx.db(), id).await)?;
        if !connection.has_credential() {
            return Ok(self.answer(false, "No credential is stored for this connection.".into()));
        }
        let stored = self.stored_credential(&connection).await?;
        let outcome = match connection.kind {
            ConnectionKind::Gitlab => self.probe.gitlab(connection.base_url.as_deref().unwrap_or_default(), &stored).await,
            ConnectionKind::Kubernetes => self.probe.kubernetes(&KubeAccess::from_stored(&stored)?).await,
            ConnectionKind::Jira => self.probe.jira(&JiraSite::of(&connection), &JiraCredential::from_stored(&stored)?).await,
        };
        Ok(self.answer(matches!(outcome, ProbeOutcome::Accepted { .. }), described(&outcome)))
    }

    /// The keychain may block on a prompt, so it is read off the async runtime.
    async fn stored_credential(&self, connection: &external_connection::Model) -> Result<Secret> {
        let secrets = self.secrets.clone();
        let account = connection.secret_ref.clone();
        tokio::task::spawn_blocking(move || secrets.read(&account))
            .await
            .map_err(|failure| RekallError::internal(InternalFailure::Background, failure))?
    }

    fn answer(&self, ok: bool, detail: String) -> ConnectionCheck {
        ConnectionCheck { ok, detail, checked_at: self.ctx.now() }
    }
}

fn described(outcome: &ProbeOutcome) -> String {
    match outcome {
        ProbeOutcome::Accepted { identity, version } => {
            let who = identity.as_deref().map_or_else(|| "The credential is accepted".to_string(), |name| format!("Signed in as {name}"));
            version.as_deref().map_or_else(|| format!("{who}."), |version| format!("{who}, server {version}."))
        }
        ProbeOutcome::Rejected { status } => format!("The server rejected the credential (HTTP {status}). Replace it."),
        ProbeOutcome::Unreachable { reason } => format!("The server could not be asked: {reason}."),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/connection/connection_check_service_tests.rs"]
mod tests;
