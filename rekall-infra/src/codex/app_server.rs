use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use rekall_kernel::{RekallError, Result};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::time::timeout;

use super::CodexCli;

pub(super) const CLI_TIMEOUT: Duration = Duration::from_secs(15);

pub(crate) struct AppServer {
    process: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
    next_id: u32,
}

#[derive(Deserialize)]
struct Reply<T> {
    result: Option<T>,
    error: Option<RpcError>,
}

#[derive(Deserialize)]
struct RpcError {
    message: String,
}

impl AppServer {
    pub(super) async fn open(cli: &CodexCli, binary: &Path) -> Result<Self> {
        let mut command = cli.command(binary).await;
        command.arg("app-server").stderr(Stdio::null());
        Self::start(command).await.map(|(server, _)| server)
    }

    pub(crate) async fn start(mut command: Command) -> Result<(Self, String)> {
        let mut process = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|failure| unavailable(failure.to_string()))?;
        let input = process.stdin.take().expect("Codex stdin was requested");
        let output = process.stdout.take().expect("Codex stdout was requested");
        let mut server = Self { process, input, output: BufReader::new(output), next_id: 0 };
        let initialized: InitializeResponse = server.request("initialize", json!({
            "clientInfo": { "name": "rekall", "version": env!("CARGO_PKG_VERSION") },
        })).await?;
        server.send(json!({ "method": "initialized" })).await?;
        Ok((server, initialized.user_agent))
    }

    pub(crate) fn into_parts(self) -> (Child, ChildStdin, BufReader<ChildStdout>, u32) {
        (self.process, self.input, self.output, self.next_id)
    }

    pub(crate) async fn request<T: DeserializeOwned>(&mut self, method: &str, params: impl serde::Serialize) -> Result<T> {
        self.next_id += 1;
        let id = self.next_id;
        timeout(CLI_TIMEOUT, async {
            self.send(json!({ "id": id, "method": method, "params": params })).await?;
            self.reply(id).await
        }).await.map_err(|_| unavailable(format!("{method} timed out")))?
    }

    async fn reply<T: DeserializeOwned>(&mut self, id: u32) -> Result<T> {
        let mut line = String::new();
        loop {
            line.clear();
            let bytes = self.output.read_line(&mut line).await.map_err(|failure| unavailable(failure.to_string()))?;
            if bytes == 0 {
                return Err(unavailable("the process closed before replying".into()));
            }
            let message: Value = serde_json::from_str(&line).map_err(|failure| unavailable(failure.to_string()))?;
            if message.get("id").and_then(Value::as_u64) == Some(u64::from(id)) && message.get("method").is_none() {
                let reply: Reply<T> = serde_json::from_value(message).map_err(|failure| unavailable(failure.to_string()))?;
                return reply.result.ok_or_else(|| unavailable(reply.error.map(|error| error.message).unwrap_or_else(|| "reply was empty".into())));
            }
        }
    }

    async fn send(&mut self, message: Value) -> Result<()> {
        let line = format!("{message}\n");
        self.input.write_all(line.as_bytes()).await.map_err(|failure| unavailable(failure.to_string()))
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct InitializeResponse {
    #[serde(default)]
    user_agent: String,
}

fn unavailable(message: String) -> RekallError {
    RekallError::conflict(format!("Codex app server: {message}"))
}
