use serde::Serialize;

use crate::chat::{ContextFill, ToolInput};

/// Longest tool output an event carries; the rest is cut and flagged.
pub const TOOL_EXCERPT_MAX_CHARACTERS: usize = 4000;

/// What a chat event says. A text or thinking event not yet `complete` carries only the words that
/// arrived since the last one, to append; the `complete` one carries the whole text, which replaces
/// what was appended. `parent_tool_use_id` is the id of the Agent call a subagent's event belongs to,
/// and `None` for the main conversation. A turn's `context` is what its last API call held, not the sum of its calls.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ChatBody {
    SessionStarted { model: String, cwd: String, permission_mode: String, version: String },
    CommandOutput { command: String, text: String },
    UserMessage { text: String },
    AssistantText { message_id: String, text: String, complete: bool, parent_tool_use_id: Option<String> },
    AssistantThinking { message_id: String, text: String, complete: bool, parent_tool_use_id: Option<String> },
    ToolUse { tool_use_id: String, name: String, input: ToolInput, parent_tool_use_id: Option<String> },
    ToolResult { tool_use_id: String, is_error: bool, excerpt: String, truncated: bool, summary: Option<String>, parent_tool_use_id: Option<String> },
    PermissionRequested { request_id: String, tool_use_id: String, name: String, input: ToolInput },
    PermissionResolved { request_id: String, allowed: bool },
    TurnFinished { usage: TurnUsage, duration_ms: u64, outcome: TurnOutcome, cost_usd: Option<f64>, context: Option<ContextFill> },
    Notice { kind: NoticeKind, text: String },
    Ended { exit_code: i32, detail: String },
}

impl ChatBody {
    /// A tool's result, its output cut to [`TOOL_EXCERPT_MAX_CHARACTERS`]; `summary` is the one line
    /// of what the tool did, when `claude` reported enough to say.
    pub fn tool_result(tool_use_id: &str, is_error: bool, output: &str, summary: Option<String>, parent_tool_use_id: Option<String>) -> Self {
        let truncated = output.chars().count() > TOOL_EXCERPT_MAX_CHARACTERS;
        let excerpt = output.chars().take(TOOL_EXCERPT_MAX_CHARACTERS).collect();
        ChatBody::ToolResult { tool_use_id: tool_use_id.to_string(), is_error, excerpt, truncated, summary, parent_tool_use_id }
    }

    pub fn notice(kind: NoticeKind, text: impl Into<String>) -> Self {
        ChatBody::Notice { kind, text: text.into() }
    }
}

/// The token counters of one turn, as Claude Code reports them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TurnUsage {
    pub input_tokens: u64,
    pub cache_creation_input_tokens: u64,
    pub cache_read_input_tokens: u64,
    pub output_tokens: u64,
}

/// How a turn ended: it answered, it was stopped by the user, or it failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TurnOutcome {
    Completed,
    Interrupted,
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum NoticeKind {
    ApiRetry,
    Compaction,
    Warning,
}

#[cfg(test)]
#[path = "../../../tests/unit/chat/model/chat_body_tests.rs"]
mod tests;
