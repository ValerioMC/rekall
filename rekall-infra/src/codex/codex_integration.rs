use std::path::Path;
use std::process::Output;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use rekall_core::codex::{CodexClient, CodexModel, CodexSetup, CodexUsage, CodexUsageWindow};
use rekall_kernel::{RekallError, Result};
use serde::Deserialize;
use serde_json::json;
use tokio::sync::Mutex;
use tokio::time::timeout;
use tracing::warn;

use super::app_server::{AppServer, CLI_TIMEOUT};
use super::rekall_skill::RekallSkill;
use super::CodexCli;

const SERVER_NAME: &str = "rekall";
const USAGE_TTL: Duration = Duration::from_secs(60);

#[derive(Clone)]
pub struct CodexIntegration {
    cli: CodexCli,
    endpoint: String,
    usage: Arc<Mutex<Option<(Instant, CodexUsage)>>>,
    install: Arc<Mutex<()>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ModelPage {
    data: Vec<CodexModel>,
    next_cursor: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RateLimitsReply {
    rate_limits: RateLimits,
}

#[derive(Deserialize)]
struct RateLimits {
    primary: Option<CodexUsageWindow>,
    secondary: Option<CodexUsageWindow>,
}

#[derive(Deserialize)]
struct Registration {
    name: String,
    enabled: bool,
    transport: RegistrationTransport,
}

#[derive(Deserialize)]
struct RegistrationTransport {
    url: Option<String>,
}

impl CodexIntegration {
    pub fn new(cli: CodexCli, endpoint: String) -> Self {
        Self { cli, endpoint, usage: Arc::new(Mutex::new(None)), install: Arc::new(Mutex::new(())) }
    }

    async fn read_usage(&self) -> CodexUsage {
        let Some(binary) = self.cli.locate().await else {
            return CodexUsage::CliMissing;
        };
        let result = async {
            let mut server = AppServer::open(&self.cli, &binary).await?;
            server.request::<RateLimitsReply>("account/rateLimits/read", json!({})).await
        }.await;
        match result {
            Ok(RateLimitsReply { rate_limits: RateLimits { primary, secondary } }) if primary.is_some() || secondary.is_some() => {
                CodexUsage::Ok { primary, secondary }
            }
            Ok(_) => CodexUsage::Unavailable,
            Err(failure) => {
                tracing::debug!(%failure, "Could not read Codex account usage");
                CodexUsage::Unavailable
            }
        }
    }

    async fn setup_for(&self, binary: &Path) -> Result<CodexSetup> {
        let output = self.run(binary, &["mcp", "list", "--json"]).await?;
        let registrations: Vec<Registration> = serde_json::from_slice(&output.stdout)
            .map_err(|failure| RekallError::conflict(format!("Could not read Codex MCP registrations: {failure}")))?;
        let registered = registrations.iter().any(|item| {
            item.name == SERVER_NAME && item.enabled && item.transport.url.as_deref() == Some(self.endpoint.as_str())
        });
        if registered && self.skill().await.installed() {
            return Ok(CodexSetup::Connected);
        }
        Ok(CodexSetup::NotConnected)
    }

    async fn run(&self, binary: &Path, args: &[&str]) -> Result<Output> {
        let output = timeout(CLI_TIMEOUT, self.cli.command(binary).await.args(args).output())
            .await
            .map_err(|_| RekallError::conflict("Codex CLI did not respond in time."))?
            .map_err(|failure| RekallError::conflict(format!("Could not run Codex CLI: {failure}")))?;
        if !output.status.success() {
            warn!(status = %output.status, "Codex refused the MCP operation");
            return Err(RekallError::conflict("Codex could not update or read its MCP configuration. Check the Codex configuration and try again."));
        }
        Ok(output)
    }

    async fn skill(&self) -> RekallSkill {
        RekallSkill::under(&self.cli.codex_home().await)
    }
}

#[async_trait]
impl CodexClient for CodexIntegration {
    async fn models(&self) -> Result<Vec<CodexModel>> {
        let binary = self.cli.require().await?;
        let mut server = AppServer::open(&self.cli, &binary).await?;
        let mut models = Vec::new();
        let mut cursor: Option<String> = None;
        loop {
            let page: ModelPage = server.request("model/list", json!({ "cursor": cursor, "limit": 50, "includeHidden": false })).await?;
            models.extend(page.data);
            cursor = page.next_cursor;
            if cursor.is_none() {
                return Ok(models);
            }
        }
    }

    async fn usage(&self, refresh: bool) -> CodexUsage {
        let mut cache = self.usage.lock().await;
        if let Some((_, usage)) = cache.as_ref().filter(|(read_at, _)| !refresh && read_at.elapsed() < USAGE_TTL) {
            return usage.clone();
        }
        let usage = self.read_usage().await;
        *cache = Some((Instant::now(), usage.clone()));
        usage
    }

    async fn setup(&self) -> Result<CodexSetup> {
        let Some(binary) = self.cli.locate().await else {
            return Ok(CodexSetup::CliMissing);
        };
        let _gate = self.install.lock().await;
        self.setup_for(&binary).await
    }

    async fn install(&self) -> Result<CodexSetup> {
        let binary = self.cli.require().await?;
        let _gate = self.install.lock().await;
        self.skill().await.install()?;
        self.run(&binary, &["mcp", "add", SERVER_NAME, "--url", &self.endpoint]).await?;
        self.setup_for(&binary).await
    }
}

#[cfg(test)]
#[path = "../../tests/unit/codex/codex_integration_tests.rs"]
mod tests;
