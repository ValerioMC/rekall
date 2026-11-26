use std::collections::BTreeMap;
use std::path::Path;

use rekall_core::chat::{ChatBody, ContextFill};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::codex::CodexTokenCounts;

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
        changes: Vec<FileUpdateChange>,
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
pub(super) struct FileUpdateChange {
    pub path: String,
    pub kind: PatchChangeKind,
    pub diff: String,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub(super) enum PatchChangeKind {
    Add,
    Delete,
    Update { move_path: Option<String> },
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
    pub total: CodexTokenCounts,
    pub last: CodexTokenCounts,
    pub model_context_window: Option<u64>,
}

impl TokenUsage {
    pub(super) fn context(self) -> Option<ContextFill> {
        let window_tokens = self.model_context_window.filter(|window| *window > 0)?;
        Some(ContextFill { used_tokens: self.last.total_tokens, window_tokens })
    }
}

pub(super) struct Approval {
    pub id: RequestId,
    pub view_id: String,
    pub event: ChatBody,
    pub reply: ApprovalReply,
}

pub(super) enum ApprovalReply {
    Decision,
    McpTool,
}

impl ApprovalReply {
    pub(super) fn result(&self, allow: bool) -> Value {
        match (self, allow) {
            (Self::Decision, true) => json!({ "decision": "accept" }),
            (Self::Decision, false) => json!({ "decision": "decline" }),
            (Self::McpTool, true) => json!({ "action": "accept", "content": {} }),
            (Self::McpTool, false) => json!({ "action": "decline" }),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct McpToolApprovalParams {
    pub thread_id: String,
    pub turn_id: String,
    pub server_name: String,
    #[serde(rename = "_meta")]
    pub tool: McpToolApproval,
    #[serde(rename = "mode")]
    _mode: ApprovalFormMode,
    requested_schema: ApprovalSchema,
}

impl McpToolApprovalParams {
    pub(super) fn is_confirmation(&self) -> bool {
        self.requested_schema.properties.is_empty()
            && self.requested_schema.required.as_ref().is_none_or(Vec::is_empty)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
enum ApprovalFormMode {
    Form,
}

#[derive(Deserialize)]
struct ApprovalSchema {
    properties: BTreeMap<String, Value>,
    required: Option<Vec<String>>,
}

#[derive(Deserialize)]
#[serde(tag = "codex_approval_kind", rename_all = "snake_case")]
pub(super) enum McpToolApproval {
    McpToolCall { tool_name: String, tool_params: Value },
}
