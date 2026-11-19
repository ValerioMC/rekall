use std::collections::HashMap;
use std::sync::Arc;

use rekall_kernel::{Id, InternalFailure, RekallError, Result};
use rekall_store::repository as repo;
use rekall_store::telemetry_setting;

use crate::claude::TerminalLaunch;
use crate::connection::{Secret, SecretStore};
use crate::telemetry::rule::{otel_variables, telemetry_rules};
use crate::telemetry::{TelemetryChange, TelemetryDestination, TelemetrySettingsView};
use crate::{in_read, in_write, Ctx};

pub const DEFAULT_HOST: &str = "http://localhost:3000";

/// The credential store account the instance's secret key is kept under.
pub const SECRET_KEY_ACCOUNT: &str = "langfuse-secret-key";

/// Where traces go: the setting is a row of the database, the secret key sits in the OS credential
/// store, and a launch gets variables only while the setting is enabled and complete.
#[derive(Clone)]
pub struct TelemetrySettingsService {
    ctx: Ctx,
    secrets: Arc<dyn SecretStore>,
}

impl TelemetrySettingsService {
    pub fn new(ctx: Ctx, secrets: Arc<dyn SecretStore>) -> Self {
        Self { ctx, secrets }
    }

    pub async fn read(&self) -> Result<TelemetrySettingsView> {
        let row = self.row().await?;
        let secret_saved = self.stored_secret().await?.is_some();
        Ok(match row {
            Some(row) => TelemetrySettingsView { enabled: row.enabled, host: row.host, public_key: row.public_key, log_content: row.log_content, secret_saved },
            None => TelemetrySettingsView { enabled: false, host: DEFAULT_HOST.to_string(), public_key: String::new(), log_content: false, secret_saved },
        })
    }

    pub async fn change(&self, change: TelemetryChange) -> Result<TelemetrySettingsView> {
        let host = telemetry_rules::host(change.host.as_deref())?;
        let public_key = telemetry_rules::public_key(change.public_key.as_deref())?;
        let secret_key = telemetry_rules::secret_key(change.secret_key.as_deref())?;
        self.save_row(change.enabled, host, public_key, change.log_content).await?;
        if let Some(secret_key) = secret_key {
            self.secrets.put(SECRET_KEY_ACCOUNT, &Secret::new(secret_key))?;
        }
        self.read().await
    }

    /// The variables a terminal opened on `launch` starts `claude` with; none while tracing is off or incomplete.
    pub async fn launch_variables(&self, launch: &TerminalLaunch) -> Result<HashMap<String, String>> {
        let Some(destination) = self.destination().await? else { return Ok(HashMap::new()) };
        Ok(otel_variables::launch_variables(&destination, &launch.project_label, launch.task_code.as_deref()))
    }

    async fn destination(&self) -> Result<Option<TelemetryDestination>> {
        let Some(row) = self.row().await?.filter(|row| row.enabled && !row.public_key.is_empty()) else { return Ok(None) };
        let Some(secret_key) = self.stored_secret().await? else { return Ok(None) };
        Ok(Some(TelemetryDestination { host: row.host, public_key: row.public_key, secret_key, log_content: row.log_content }))
    }

    async fn row(&self) -> Result<Option<telemetry_setting::Model>> {
        in_read!(&self.ctx, |tx| repo::telemetry_setting::find(tx.db()).await)
    }

    async fn save_row(&self, enabled: bool, host: String, public_key: String, log_content: bool) -> Result<()> {
        let now = self.ctx.now();
        in_write!(&self.ctx, |tx| {
            let Some(before) = repo::telemetry_setting::find(tx.db()).await? else {
                let first = telemetry_setting::Model { id: Id::random(), enabled, host, public_key, log_content, created_at: now, updated_at: now };
                return repo::telemetry_setting::insert(tx.db(), first).await.map(|_| ());
            };
            let changed = telemetry_setting::Model { enabled, host, public_key, log_content, ..before.clone() };
            repo::update_if_changed(tx.db(), &before, changed, now).await.map(|_| ())
        })
    }

    async fn stored_secret(&self) -> Result<Option<Secret>> {
        let secrets = self.secrets.clone();
        let read = tokio::task::spawn_blocking(move || secrets.read(SECRET_KEY_ACCOUNT))
            .await
            .map_err(|failure| RekallError::internal(InternalFailure::Background, failure))?;
        match read {
            Ok(secret) => Ok(Some(secret)),
            Err(RekallError::NotFound(_)) => Ok(None),
            Err(failure) => Err(failure),
        }
    }
}
