use std::path::{Path, PathBuf};

use rekall_kernel::Result;
use serde::Deserialize;
use serde_json::json;

use super::{app_server::AppServer, CodexCli};

#[derive(Deserialize)]
pub(crate) struct CodexHookReport {
    data: Vec<DirectoryHooks>,
}

#[derive(Deserialize)]
struct DirectoryHooks {
    hooks: Vec<CodexHook>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CodexHook {
    command: Option<String>,
    source_path: PathBuf,
    pub enabled: bool,
    pub trust_status: HookTrust,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum HookTrust {
    Trusted,
    Managed,
    Untrusted,
    Modified,
    #[serde(other)]
    Unknown,
}

impl CodexHookReport {
    pub(crate) async fn read(cli: &CodexCli, binary: &Path) -> Result<Self> {
        let mut server = AppServer::open(cli, binary).await?;
        server.request("hooks/list", json!({})).await
    }

    pub(crate) fn find(&self, source: &Path, command: &str) -> Option<&CodexHook> {
        self.data
            .iter()
            .flat_map(|entry| &entry.hooks)
            .find(|hook| hook.source_path == source && hook.command.as_deref() == Some(command))
    }
}
