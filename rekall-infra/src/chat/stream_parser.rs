use std::collections::HashMap;

use rekall_core::chat::{ChatBody, NoticeKind, ToolInput, TurnOutcome, TurnUsage};
use tracing::warn;

use super::tool_summary::summarise;
use super::wire::{WireBlock, WireBlockStart, WireControlRequest, WireDelta, WireLine, WireLocalCommand, WireMessage, WireResult, WireStreamEvent, WireSystem, WireToolUseResult};

const LOGGED_LINE_CHARACTERS: usize = 200;
const ABORTED_PREFIX: &str = "aborted";
const INIT_SUBTYPE: &str = "init";
const COMMAND_OUTPUT_TAGS: [&str; 2] = ["local-command-stdout", "local-command-stderr"];

/// Reads the stdout of a headless `claude`, one line at a time, into the events the window follows.
/// A message's text arrives as deltas while it streams and whole when its block ends; the parser
/// keeps which message the deltas belong to and joins the blocks of one message. `claude` repeats
/// `init` on every turn, so the session is announced again only when it changed.
#[derive(Default)]
pub struct StreamParser {
    streaming_message: Option<String>,
    texts: HashMap<String, String>,
    announced_session: Option<ChatBody>,
}

impl StreamParser {
    pub fn new() -> Self {
        Self::default()
    }

    /// The events one line means: none for a line Rekall does not show, and a warning notice for
    /// one it cannot read.
    pub fn parse_line(&mut self, line: &str) -> Vec<ChatBody> {
        let line = line.trim();
        if line.is_empty() {
            return Vec::new();
        }
        match serde_json::from_str::<WireLine>(line) {
            Ok(parsed) => self.read(parsed),
            Err(failure) => vec![unreadable(line, &failure)],
        }
    }

    fn read(&mut self, line: WireLine) -> Vec<ChatBody> {
        match line {
            WireLine::System(system) => self.system(&system),
            WireLine::StreamEvent { event, parent_tool_use_id } => self.stream(event, parent_tool_use_id),
            WireLine::Assistant { local_command_source: Some(output), local_command_run, .. } => {
                command_output(&output, local_command_run).into_iter().collect()
            }
            WireLine::Assistant { message, parent_tool_use_id, .. } => self.assistant(message, parent_tool_use_id),
            WireLine::User { message, is_replay, tool_use_result, parent_tool_use_id } if !is_replay => {
                tool_results(message, tool_use_result.as_ref(), parent_tool_use_id)
            }
            WireLine::User { .. } => Vec::new(),
            WireLine::Result(result) => self.finish_turn(result),
            WireLine::ControlRequest { request_id, request } => permission_request(request_id, request).into_iter().collect(),
            WireLine::ConversationReset {} => vec![ChatBody::notice(NoticeKind::Warning, "The conversation was cleared.")],
            WireLine::Other => Vec::new(),
        }
    }

    fn system(&mut self, system: &WireSystem) -> Vec<ChatBody> {
        if system.subtype == INIT_SUBTYPE {
            return self.announce_session(system).into_iter().collect();
        }
        system_notice(system).into_iter().collect()
    }

    fn announce_session(&mut self, system: &WireSystem) -> Option<ChatBody> {
        let started = session_started(system)?;
        if self.announced_session.as_ref() == Some(&started) {
            return None;
        }
        self.announced_session = Some(started.clone());
        Some(started)
    }

    fn stream(&mut self, event: WireStreamEvent, parent_tool_use_id: Option<String>) -> Vec<ChatBody> {
        match event {
            WireStreamEvent::MessageStart { message } => {
                self.streaming_message = Some(message.id);
                Vec::new()
            }
            WireStreamEvent::ContentBlockStart { content_block: WireBlockStart::Thinking } => {
                self.partial_thinking(String::new(), parent_tool_use_id)
            }
            WireStreamEvent::ContentBlockDelta { delta: WireDelta::TextDelta { text } } => self.partial_text(text, parent_tool_use_id),
            WireStreamEvent::ContentBlockDelta { delta: WireDelta::ThinkingDelta { thinking } } if !thinking.is_empty() => {
                self.partial_thinking(thinking, parent_tool_use_id)
            }
            _ => Vec::new(),
        }
    }

    fn partial_text(&self, text: String, parent_tool_use_id: Option<String>) -> Vec<ChatBody> {
        match &self.streaming_message {
            Some(message_id) => vec![ChatBody::AssistantText { message_id: message_id.clone(), text, complete: false, parent_tool_use_id }],
            None => Vec::new(),
        }
    }

    fn partial_thinking(&self, text: String, parent_tool_use_id: Option<String>) -> Vec<ChatBody> {
        match &self.streaming_message {
            Some(message_id) => vec![ChatBody::AssistantThinking { message_id: message_id.clone(), text, complete: false, parent_tool_use_id }],
            None => Vec::new(),
        }
    }

    fn assistant(&mut self, message: WireMessage, parent_tool_use_id: Option<String>) -> Vec<ChatBody> {
        let message_id = message.id.clone().or_else(|| self.streaming_message.clone()).unwrap_or_default();
        message.content.0.into_iter().filter_map(|block| self.block_event(&message_id, parent_tool_use_id.as_deref(), block)).collect()
    }

    fn block_event(&mut self, message_id: &str, parent_tool_use_id: Option<&str>, block: WireBlock) -> Option<ChatBody> {
        let parent_tool_use_id = parent_tool_use_id.map(str::to_string);
        match block {
            WireBlock::Text { text } if !text.is_empty() => Some(ChatBody::AssistantText {
                message_id: message_id.to_string(),
                text: self.join_text(message_id, text),
                complete: true,
                parent_tool_use_id,
            }),
            WireBlock::Thinking { thinking } => {
                Some(ChatBody::AssistantThinking { message_id: message_id.to_string(), text: thinking, complete: true, parent_tool_use_id })
            }
            WireBlock::ToolUse { id, name, input } => {
                let input = ToolInput::of(&name, &input);
                Some(ChatBody::ToolUse { tool_use_id: id, name, input, parent_tool_use_id })
            }
            _ => None,
        }
    }

    fn join_text(&mut self, message_id: &str, text: String) -> String {
        let whole = self.texts.entry(message_id.to_string()).or_default();
        if !whole.is_empty() {
            whole.push_str("\n\n");
        }
        whole.push_str(&text);
        whole.clone()
    }

    fn finish_turn(&mut self, result: WireResult) -> Vec<ChatBody> {
        self.streaming_message = None;
        self.texts.clear();
        let usage = result.usage.unwrap_or_default();
        vec![ChatBody::TurnFinished {
            usage: TurnUsage {
                input_tokens: usage.input_tokens,
                cache_creation_input_tokens: usage.cache_creation_input_tokens,
                cache_read_input_tokens: usage.cache_read_input_tokens,
                output_tokens: usage.output_tokens,
            },
            duration_ms: result.duration_ms.unwrap_or_default(),
            outcome: outcome_of(result.is_error, result.terminal_reason.as_deref()),
        }]
    }
}

fn outcome_of(is_error: bool, terminal_reason: Option<&str>) -> TurnOutcome {
    if terminal_reason.is_some_and(|reason| reason.starts_with(ABORTED_PREFIX)) {
        return TurnOutcome::Interrupted;
    }
    if is_error {
        return TurnOutcome::Failed;
    }
    TurnOutcome::Completed
}

fn tool_results(message: WireMessage, reported: Option<&WireToolUseResult>, parent_tool_use_id: Option<String>) -> Vec<ChatBody> {
    let results: Vec<_> = message
        .content
        .0
        .into_iter()
        .filter_map(|block| match block {
            WireBlock::ToolResult { tool_use_id, content, is_error } => Some((tool_use_id, content, is_error)),
            _ => None,
        })
        .collect();
    let summary = if results.len() == 1 { reported.and_then(summarise) } else { None };
    results
        .into_iter()
        .map(|(tool_use_id, content, is_error)| {
            let output = content.map(|content| content.text()).unwrap_or_default();
            ChatBody::tool_result(&tool_use_id, is_error, &output, summary.clone(), parent_tool_use_id.clone())
        })
        .collect()
}

fn permission_request(request_id: String, request: WireControlRequest) -> Option<ChatBody> {
    match request {
        WireControlRequest::CanUseTool { tool_name, input, tool_use_id } => Some(ChatBody::PermissionRequested {
            request_id,
            tool_use_id: tool_use_id.unwrap_or_default(),
            input: ToolInput::of(&tool_name, &input),
            name: tool_name,
        }),
        WireControlRequest::Other => None,
    }
}

fn system_notice(system: &WireSystem) -> Option<ChatBody> {
    match system.subtype.as_str() {
        "api_retry" => {
            let seconds = system.retry_delay_ms.unwrap_or_default().div_ceil(1000);
            Some(ChatBody::notice(
                NoticeKind::ApiRetry,
                format!("The API did not answer. Retry {} in {seconds}s.", system.attempt.unwrap_or(1)),
            ))
        }
        "compact_boundary" => Some(ChatBody::notice(NoticeKind::Compaction, "The conversation was compacted.")),
        _ => None,
    }
}

fn session_started(system: &WireSystem) -> Option<ChatBody> {
    Some(ChatBody::SessionStarted {
        model: system.model.clone()?,
        cwd: system.cwd.clone()?,
        permission_mode: system.permission_mode.clone()?,
        version: system.claude_code_version.clone()?,
    })
}

fn command_output(source: &str, run: Option<WireLocalCommand>) -> Option<ChatBody> {
    let text = untag_command_output(source);
    if text.is_empty() {
        return None;
    }
    Some(ChatBody::CommandOutput { command: command_label(run), text: text.to_string() })
}

fn untag_command_output(source: &str) -> &str {
    let source = source.trim();
    COMMAND_OUTPUT_TAGS
        .iter()
        .find_map(|tag| source.strip_prefix(&format!("<{tag}>"))?.strip_suffix(&format!("</{tag}>")))
        .unwrap_or(source)
        .trim()
}

fn command_label(run: Option<WireLocalCommand>) -> String {
    match run {
        Some(run) => format!("/{} {}", run.command, run.args).trim().to_string(),
        None => String::new(),
    }
}

fn unreadable(line: &str, failure: &serde_json::Error) -> ChatBody {
    let excerpt: String = line.chars().take(LOGGED_LINE_CHARACTERS).collect();
    warn!(%failure, line = %excerpt, "Could not read a line claude printed");
    ChatBody::notice(NoticeKind::Warning, "Skipped a line from claude that Rekall could not read.")
}

#[cfg(test)]
#[path = "../../tests/unit/chat/stream_parser_tests.rs"]
mod tests;
