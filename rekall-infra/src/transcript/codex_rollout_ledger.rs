use std::collections::HashMap;

use rekall_core::token::{UnitUsage, WorkUnit};
use rekall_kernel::Instant;
use serde::Deserialize;

use crate::codex::CodexTokenCounts;

use super::transcript_entry::ToolInput;
use super::work_terms::{unit_of_anchors, unit_of_prompt};

#[derive(Default)]
pub(super) struct CodexRolloutLedger {
    current: Option<WorkUnit>,
    pending: HashMap<String, PendingMove>,
    after_reading: Option<WorkUnit>,
    previous: CodexTokenCounts,
    spent: Vec<UnitUsage>,
}

struct PendingMove {
    unit: WorkUnit,
    confirmation: Confirmation,
    previous_total: u64,
}

enum Confirmation {
    Context,
    Step(&'static str),
}

#[derive(Deserialize)]
struct RolloutLine {
    timestamp: Option<Instant>,
    #[serde(flatten)]
    event: RolloutEvent,
}

#[derive(Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
enum RolloutEvent {
    EventMsg(EventMessage),
    ResponseItem(ResponseItem),
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum EventMessage {
    TokenCount { info: Option<TokenInfo> },
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
struct TokenInfo {
    total_token_usage: CodexTokenCounts,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ResponseItem {
    Message { role: String, content: Vec<MessageContent> },
    FunctionCall { name: String, namespace: Option<String>, call_id: String, arguments: String },
    FunctionCallOutput { call_id: String, output: Option<CallOutput> },
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum CallOutput {
    Annotated { success: bool },
    Text(String),
    Blocks(Vec<MessageContent>),
    Other(serde::de::IgnoredAny),
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum MessageContent {
    InputText { text: String },
    #[serde(other)]
    Other,
}

impl CodexRolloutLedger {
    pub(super) fn read_line(&mut self, line: &str) {
        let Ok(line) = serde_json::from_str::<RolloutLine>(line) else { return };
        match line.event {
            RolloutEvent::EventMsg(EventMessage::TokenCount { info }) => {
                if let Some(info) = info {
                    self.charge(info, line.timestamp);
                }
                if let Some(unit) = self.after_reading.take() {
                    self.current = Some(unit);
                }
            }
            RolloutEvent::ResponseItem(item) => self.read_item(item),
            _ => {}
        }
    }

    pub(super) fn spent(self) -> Vec<UnitUsage> {
        self.spent
    }

    fn read_item(&mut self, item: ResponseItem) {
        match item {
            ResponseItem::Message { role, content } if role == "user" => {
                for block in content {
                    if let MessageContent::InputText { text } = block {
                        if let Some(unit) = unit_of_prompt(&text) {
                            self.current = Some(self.continued(unit));
                        }
                    }
                }
            }
            ResponseItem::FunctionCall { name, namespace, call_id, arguments } => {
                if let Some((unit, confirmation)) = self.move_from_call(&name, namespace.as_deref(), &arguments) {
                    self.pending.insert(call_id, PendingMove { unit, confirmation, previous_total: self.previous.total_tokens });
                }
            }
            ResponseItem::FunctionCallOutput { call_id, output } => {
                if let Some(movement) = self.pending.remove(&call_id) {
                    self.confirm_move(movement, output);
                }
            }
            _ => {}
        }
    }

    fn move_from_call(&self, name: &str, namespace: Option<&str>, arguments: &str) -> Option<(WorkUnit, Confirmation)> {
        if namespace.is_some_and(|name| !matches!(name, "mcp__rekall" | "mcp__rekall__" | "rekall")) {
            return None;
        }
        let name = name.strip_prefix("mcp__rekall__").unwrap_or(name);
        if !matches!(name, "rekall_context" | "rekall_step") {
            return None;
        }
        let input: ToolInput = serde_json::from_str(arguments).ok()?;
        let unit = unit_of_anchors(input.anchors.as_deref()?)?;
        match name {
            "rekall_context" => Some((self.continued(unit), Confirmation::Context)),
            _ => match input.state.as_deref()?.trim().to_ascii_lowercase().as_str() {
                "running" | "start" => Some((unit.on_step(input.step.map(|step| step.text()).as_deref()), Confirmation::Step("running"))),
                "claimed" | "done" => Some((unit, Confirmation::Step("claimed"))),
                "open" => Some((unit, Confirmation::Step("open"))),
                _ => None,
            },
        }
    }

    /// A rollout can persist a tool result before the tokens spent requesting it.
    fn confirm_move(&mut self, movement: PendingMove, output: Option<CallOutput>) {
        let Some(output) = output else { return };
        if !movement.confirmation.matches(output) {
            return;
        }
        if self.previous.total_tokens > movement.previous_total {
            self.current = Some(movement.unit);
        } else {
            self.after_reading = Some(movement.unit);
        }
    }

    fn continued(&self, unit: WorkUnit) -> WorkUnit {
        match &self.current {
            Some(current) if unit.step.is_none() && current.same_task(&unit) => current.clone(),
            _ => unit,
        }
    }

    fn charge(&mut self, info: TokenInfo, at: Option<Instant>) {
        let total = info.total_token_usage;
        if total.total_tokens <= self.previous.total_tokens {
            return;
        }
        let tokens = total.since(self.previous);
        self.previous = total;
        if tokens.is_empty() {
            return;
        }
        let Some(unit) = &self.current else { return };
        let day = at.map(|time| time.start_of_day());
        if let Some(existing) = self.spent.iter_mut().find(|usage| usage.unit == *unit && usage.at == day) {
            existing.tokens += tokens;
        } else {
            self.spent.push(UnitUsage { unit: unit.clone(), at: day, tokens });
        }
    }
}

impl Confirmation {
    fn matches(&self, output: CallOutput) -> bool {
        match output {
            CallOutput::Annotated { success } => success,
            CallOutput::Text(text) => self.matches_text(&text),
            CallOutput::Blocks(blocks) => blocks.iter().any(|block| match block {
                MessageContent::InputText { text } => self.matches_text(text),
                MessageContent::Other => false,
            }),
            CallOutput::Other(_) => false,
        }
    }

    fn matches_text(&self, text: &str) -> bool {
        match self {
            Self::Context => text.trim_start().starts_with("# Context"),
            Self::Step(state) => text.lines().next().is_some_and(|line| {
                line.starts_with("Step ") && line.ends_with(&format!(" is now `{state}`."))
            }),
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/transcript/codex_rollout_ledger_tests.rs"]
mod tests;
