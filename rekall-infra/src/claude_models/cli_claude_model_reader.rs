use std::fmt::Display;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use async_trait::async_trait;
use rekall_core::claude::{ClaudeModel, ClaudeModelReader};
use rekall_kernel::{InternalFailure, RekallError, Result};
use serde::Deserialize;
use serde_json::json;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::time::timeout;
use tracing::warn;

use crate::cli::ClaudeCli;

const ANSWER_TIMEOUT: Duration = Duration::from_secs(15);
const REQUEST_ID: &str = "rekall-models";
/// Stream JSON with no prompt, tools, saved session, MCP server or customisation; login and model
/// selection stay the CLI's own.
const CATALOG_ARGUMENTS: [&str; 11] = [
    "-p",
    "--verbose",
    "--input-format",
    "stream-json",
    "--output-format",
    "stream-json",
    "--no-session-persistence",
    "--strict-mcp-config",
    "--safe-mode",
    "--tools",
    "",
];

/// Starts one process per read in an empty folder, sends it only the `initialize` control message
/// and kills it once it answers or runs out of time.
pub struct CliClaudeModelReader {
    cli: ClaudeCli,
    answer_timeout: Duration,
}

impl CliClaudeModelReader {
    pub fn new(cli: ClaudeCli) -> Self {
        Self { cli, answer_timeout: ANSWER_TIMEOUT }
    }

    pub fn with_answer_timeout(mut self, answer_timeout: Duration) -> Self {
        self.answer_timeout = answer_timeout;
        self
    }

    async fn start_in(&self, folder: &Path) -> Result<Child> {
        let binary = self.cli.locate().await.ok_or_else(|| unavailable("its command line tool isn't on this machine"))?;
        Command::new(binary)
            .args(CATALOG_ARGUMENTS)
            .current_dir(folder)
            .env_clear()
            .envs(self.cli.environment().await)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(unavailable)
    }
}

#[async_trait]
impl ClaudeModelReader for CliClaudeModelReader {
    async fn models(&self) -> Result<Vec<ClaudeModel>> {
        let neutral_folder = tempfile::tempdir().map_err(|failure| RekallError::internal(InternalFailure::Io, failure))?;
        let mut process = self.start_in(neutral_folder.path()).await?;
        let answer = timeout(self.answer_timeout, initialize(&mut process)).await;
        release(process).await;
        answer.map_err(|_| unavailable("it did not answer in time"))?
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Message {
    ControlResponse {
        response: ControlResponse,
    },
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
struct ControlResponse {
    request_id: String,
    response: Option<Initialization>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct Initialization {
    models: Vec<ClaudeModel>,
}

impl ControlResponse {
    fn into_models(self) -> Result<Vec<ClaudeModel>> {
        match self.response {
            Some(initialization) => Ok(initialization.models),
            None => Err(unavailable(self.error.unwrap_or_else(|| "its answer was empty".into()))),
        }
    }
}

async fn initialize(process: &mut Child) -> Result<Vec<ClaudeModel>> {
    let mut input = process.stdin.take().expect("Claude Code stdin was requested");
    let output = process.stdout.take().expect("Claude Code stdout was requested");
    let request = json!({ "type": "control_request", "request_id": REQUEST_ID, "request": { "subtype": "initialize" } });
    input.write_all(format!("{request}\n").as_bytes()).await.map_err(unavailable)?;
    let mut lines = BufReader::new(output).lines();
    while let Some(line) = lines.next_line().await.map_err(unavailable)? {
        match serde_json::from_str(&line).map_err(unavailable)? {
            Message::ControlResponse { response } if response.request_id == REQUEST_ID => return response.into_models(),
            Message::ControlResponse { .. } | Message::Other => {}
        }
    }
    Err(unavailable("it closed before answering"))
}

async fn release(mut process: Child) {
    if let Err(failed) = process.kill().await {
        warn!(%failed, "Could not release the Claude Code model catalog process");
    }
}

fn unavailable(reason: impl Display) -> RekallError {
    RekallError::conflict(format!("Claude Code models are unavailable: {reason}"))
}

#[cfg(test)]
#[path = "../../tests/unit/claude_models/cli_claude_model_reader_tests.rs"]
mod tests;
