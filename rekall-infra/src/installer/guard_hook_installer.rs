use std::path::{Path, PathBuf};

use async_trait::async_trait;
use rekall_core::claude::SessionProvider;
use rekall_core::guard::{GuardInstaller, GuardProtection, GuardProtectionState};
use rekall_kernel::{InternalFailure, RekallError, Result};
use tracing::warn;

use crate::codex::{CodexCli, CodexHookReport, HookTrust};

use super::guard_hook_file::GuardHookFile;

#[derive(Clone)]
pub struct GuardHookInstaller {
    location: HookLocation,
    program: PathBuf,
}

#[derive(Clone)]
enum HookLocation {
    Claude(PathBuf),
    Codex(CodexCli),
}

impl GuardHookInstaller {
    pub fn claude(home: &Path, program: PathBuf) -> Self {
        Self {
            location: HookLocation::Claude(home.join("settings.json")),
            program,
        }
    }

    pub fn codex(cli: CodexCli, program: PathBuf) -> Self {
        Self {
            location: HookLocation::Codex(cli),
            program,
        }
    }

    async fn file(&self) -> GuardHookFile {
        let (path, provider) = match &self.location {
            HookLocation::Claude(path) => (path.clone(), SessionProvider::Claude),
            HookLocation::Codex(cli) => (cli.codex_home().await.join("hooks.json"), SessionProvider::Codex),
        };
        GuardHookFile::new(path, self.program.clone(), provider)
    }

    async fn codex_report(&self) -> std::result::Result<Option<CodexHookReport>, String> {
        let HookLocation::Codex(cli) = &self.location else { return Ok(None) };
        let binary = cli
            .locate()
            .await
            .ok_or_else(|| "Install the Codex CLI to use these hooks.".to_string())?;
        CodexHookReport::read(cli, &binary).await.map(Some).map_err(|failure| {
            warn!(%failure, "Could not inspect Codex hooks");
            "Codex could not report hook status. Update the CLI or check its configuration, then refresh.".into()
        })
    }

    async fn inspect(&self, file: &GuardHookFile, protection: GuardProtection) -> GuardProtection {
        if protection.state == GuardProtectionState::InvalidSettings {
            return protection;
        }
        match self.codex_report().await {
            Ok(Some(report)) if protection.state == GuardProtectionState::Installed => {
                let source = match tokio::fs::canonicalize(&file.path).await {
                    Ok(source) => source,
                    Err(failure) => {
                        warn!(%failure, path = %file.path.display(), "Could not locate the installed Codex hooks");
                        return file.protection(
                            GuardProtectionState::NeedsRepair,
                            true,
                            Some("The Codex hook file could not be read. Refresh to check whether it moved.".into()),
                        );
                    }
                };
                codex_protection(file, &report, &source)
            }
            Ok(_) => protection,
            Err(problem) => file.protection(GuardProtectionState::Unsupported, protection.configured, Some(problem)),
        }
    }
}

#[async_trait]
impl GuardInstaller for GuardHookInstaller {
    async fn status(&self) -> Result<GuardProtection> {
        let file = self.file().await;
        let protection = edit_guard_file(file.clone(), |file| Ok(file.status())).await?;
        Ok(self.inspect(&file, protection).await)
    }

    async fn install(&self) -> Result<GuardProtection> {
        self.codex_report().await.map_err(RekallError::conflict)?;
        let file = self.file().await;
        let protection = edit_guard_file(file.clone(), GuardHookFile::install).await?;
        Ok(self.inspect(&file, protection).await)
    }

    async fn uninstall(&self) -> Result<GuardProtection> {
        let file = self.file().await;
        let protection = edit_guard_file(file.clone(), GuardHookFile::uninstall).await?;
        Ok(self.inspect(&file, protection).await)
    }
}

async fn edit_guard_file(
    file: GuardHookFile,
    action: impl FnOnce(&GuardHookFile) -> Result<GuardProtection> + Send + 'static,
) -> Result<GuardProtection> {
    tokio::task::spawn_blocking(move || action(&file))
        .await
        .map_err(|failure| RekallError::internal(InternalFailure::Background, failure))?
}

fn codex_protection(file: &GuardHookFile, report: &CodexHookReport, source: &Path) -> GuardProtection {
    let commands = file.commands();
    let hooks = commands.iter().map(|command| report.find(source, command));
    let hooks: Option<Vec<_>> = hooks.collect();
    let Some(hooks) = hooks.filter(|hooks| hooks.iter().all(|hook| hook.enabled)) else {
        return file.protection(
            GuardProtectionState::Disabled,
            true,
            Some("Enable hooks in Codex settings and check that both Rekall entries are enabled in /hooks, then refresh.".into()),
        );
    };
    if hooks
        .iter()
        .any(|hook| !matches!(hook.trust_status, HookTrust::Trusted | HookTrust::Managed))
    {
        return file.protection(
            GuardProtectionState::TrustRequired,
            true,
            Some("Open /hooks in the Codex terminal and trust both Rekall hooks, then refresh here.".into()),
        );
    }
    file.protection(GuardProtectionState::Installed, true, None)
}

#[cfg(test)]
#[path = "../../tests/unit/installer/guard_hook_installer_tests.rs"]
mod tests;
