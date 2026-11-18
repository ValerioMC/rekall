use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use rekall_core::claude::{SessionProvider, TerminalLaunch};
use rekall_core::telemetry::TelemetrySettingsService;
use rekall_kernel::{Id, RekallError, Result};
use tracing::{info, warn};

use super::SessionOptions;
use crate::cli::ClaudeCli;
use crate::connection::CredentialBroker;

#[derive(Clone)]
pub struct SessionLauncher {
    cli: ClaudeCli,
    broker: Arc<CredentialBroker>,
    telemetry: TelemetrySettingsService,
}

impl SessionLauncher {
    pub fn new(cli: ClaudeCli, broker: Arc<CredentialBroker>, telemetry: TelemetrySettingsService) -> Self {
        Self { cli, broker, telemetry }
    }

    pub async fn require_terminal_cli(&self, provider: SessionProvider) -> Result<PathBuf> {
        match provider {
            SessionProvider::Claude => self.require_cli().await,
            SessionProvider::Codex => self.cli.codex().require().await,
        }
    }

    pub async fn require_cli(&self) -> Result<PathBuf> {
        self.cli.locate().await.ok_or_else(|| {
            RekallError::conflict("Claude Code's command line tool isn't on this machine. Install it, then try again.")
        })
    }

    /// The environment session `id` runs in. A credential session's settings land in `options`.
    pub async fn environment(&self, id: Id, launch: &TerminalLaunch, options: &mut SessionOptions) -> Result<HashMap<String, String>> {
        let mut environment = match options.provider {
            SessionProvider::Claude => self.cli.environment().await,
            SessionProvider::Codex => self.cli.codex().environment().await,
        };
        if let Some(session) = self.broker.open(id, launch, options.provider).await? {
            session.apply(&mut environment);
            options.settings = session.claude_settings;
            options.codex_credential_session = options.provider == SessionProvider::Codex;
        }
        if options.provider == SessionProvider::Claude {
            environment.extend(self.telemetry_variables(id, launch).await);
        }
        Ok(environment)
    }

    /// Ends the credential session `id` opened, if it opened one.
    pub fn release(&self, id: Id) {
        self.broker.close(id);
    }

    /// A failing settings read costs the session its traces and nothing else.
    async fn telemetry_variables(&self, terminal_id: Id, launch: &TerminalLaunch) -> HashMap<String, String> {
        match self.telemetry.launch_variables(launch).await {
            Ok(variables) => {
                if let Some(endpoint) = variables.get("OTEL_EXPORTER_OTLP_ENDPOINT") {
                    info!(%terminal_id, %endpoint, "Tracing the terminal's session");
                }
                variables
            }
            Err(failure) => {
                warn!(%failure, %terminal_id, "Could not read the tracing settings; the session starts without traces");
                HashMap::new()
            }
        }
    }
}

/// The project's folder, which has to be there for a session to start in it.
pub fn require_folder(launch: &TerminalLaunch) -> Result<&Path> {
    let directory = Path::new(&launch.working_dir);
    if !directory.is_dir() {
        return Err(RekallError::conflict(format!(
            "This project's folder ({}) is not there. Set it again on the project page.",
            launch.working_dir
        )));
    }
    Ok(directory)
}
