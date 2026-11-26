use std::path::Path;

use rekall_core::chat::{ChatBody, ContextFill, TurnUsage};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ThreadStart<'a> {
    pub cwd: &'a Path,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_policy: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox: Option<&'static str>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ThreadStarted {
    pub thread: Thread,
    pub model: String,
    pub cwd: String,
    pub approval_policy: ApprovalPolicy,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub(super) enum ApprovalPolicy {
    Named(String),
    Granular { granular: GranularPolicy },
}

#[derive(Deserialize)]
pub(super) struct GranularPolicy {}

impl ApprovalPolicy {
    pub(super) fn label(self) -> String {
        match self {
            Self::Named(name) => name,
            Self::Granular { granular: GranularPolicy {} } => "granular".to_owned(),
        }
    }
}

#[derive(Deserialize)]
pub(super) struct Thread {
    pub id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct TurnStart<'a> {
    pub thread_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<&'a str>,
    pub input: [TextInput<'a>; 1],
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub(super) enum TextInput<'a> {
    Text { text: &'a str, text_elements: [(); 0] },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct TurnInterrupt<'a> {
    pub thread_id: &'a str,
    pub turn_id: &'a str,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub(super) enum RequestId {
    Number(i64),
    Text(String),
}

#[derive(Deserialize)]
#[serde(untagged)]
pub(super) enum ServerMessage {
    Event { method: String, params: Option<Value>, id: Option<RequestId> },
    Reply { id: u32, result: Option<ReplyResult>, error: Option<RpcError> },
}

#[derive(Deserialize)]
pub(super) struct ReplyResult {
    pub turn: Option<Turn>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct TurnNotification {
    #[serde(default)]
    pub thread_id: Option<String>,
    pub turn: Turn,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Turn {
    pub id: String,
    pub status: TurnStatus,
    pub duration_ms: Option<u64>,
    pub error: Option<RpcError>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) enum TurnStatus {
    Completed,
    Interrupted,
    Failed,
    InProgress,
}

#[derive(Deserialize)]
pub(super) struct RpcError {
    pub message: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RetryNotice {
    pub thread_id: String,
    pub error: RpcError,
    pub will_retry: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Delta {
    pub thread_id: String,
    pub item_id: String,
    pub delta: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ItemNotification {
    pub thread_id: String,
    pub item: Item,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub(super) enum Item {
    AgentMessage {
        id: String,
        text: String,
    },
    Plan {
        id: String,
        text: String,
    },
    Reasoning {
        id: String,
        summary: Vec<String>,
    },
    CommandExecution {
        id: String,
        command: String,
        status: ToolStatus,
        aggregated_output: Option<String>,
        exit_code: Option<i32>,
    },
    FileChange {
        id: String,
        changes: Vec<FileChange>,
        status: ToolStatus,
    },
    McpToolCall {
        id: String,
        server: String,
        tool: String,
        arguments: Value,
        status: ToolStatus,
        result: Option<Value>,
        error: Option<RpcError>,
    },
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
pub(super) struct FileChange {
    pub path: String,
    pub diff: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) enum ToolStatus {
    Completed,
    Failed,
    Declined,
    #[serde(other)]
    Other,
}

impl ToolStatus {
    pub(super) fn failed(&self) -> bool {
        matches!(self, Self::Failed | Self::Declined)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ApprovalParams {
    pub thread_id: String,
    pub turn_id: String,
    pub item_id: String,
    pub reason: Option<String>,
    pub command: Option<String>,
    pub grant_root: Option<String>,
    pub network_approval_context: Option<NetworkApprovalContext>,
}

#[derive(Deserialize)]
pub(super) struct NetworkApprovalContext {
    pub host: String,
    pub protocol: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ApprovalResolved {
    pub thread_id: String,
    pub request_id: RequestId,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct TokenUsageUpdated {
    pub thread_id: String,
    pub turn_id: String,
    pub token_usage: TokenUsage,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct TokenUsage {
    pub total: UsageBreakdown,
    pub last: UsageBreakdown,
    pub model_context_window: Option<u64>,
}

#[derive(Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct UsageBreakdown {
    total_tokens: u64,
    input_tokens: u64,
    cached_input_tokens: u64,
    #[serde(default)]
    cache_write_input_tokens: u64,
    output_tokens: u64,
}

impl TokenUsage {
    pub(super) fn context(self) -> Option<ContextFill> {
        let window_tokens = self.model_context_window.filter(|window| *window > 0)?;
        Some(ContextFill { used_tokens: self.last.total_tokens, window_tokens })
    }
}

impl UsageBreakdown {
    pub(super) fn since(self, previous: Self) -> TurnUsage {
        let cache_read_input_tokens = self.cached_input_tokens.saturating_sub(previous.cached_input_tokens);
        let cache_creation_input_tokens = self.cache_write_input_tokens.saturating_sub(previous.cache_write_input_tokens);
        TurnUsage {
            input_tokens: self.input_tokens.saturating_sub(previous.input_tokens).saturating_sub(cache_read_input_tokens),
            cache_creation_input_tokens,
            cache_read_input_tokens,
            output_tokens: self.output_tokens.saturating_sub(previous.output_tokens),
        }
    }
}

pub(super) struct Approval {
    pub id: RequestId,
    pub view_id: String,
    pub event: ChatBody,
}
