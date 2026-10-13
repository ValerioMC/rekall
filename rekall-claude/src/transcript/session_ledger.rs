use std::collections::HashMap;

use rekall_common::Instant;
use rekall_service::token::{TokenCount, UnitUsage, WorkUnit};

use super::transcript_entry::{ToolInput, TranscriptEntry};
use super::work_terms::{unit_of_anchors, unit_of_prompt};

/// What one session spent on each unit of work. A call is charged to the unit current when it was
/// made: `/rk` or `rekall_context` moves to a task, `rekall_step` `running` to that step, `claimed`
/// or `open` back to the task. Calls before any of these are not Rekall work.
#[derive(Default)]
pub struct SessionLedger {
    current: Option<WorkUnit>,
    /// When the session moved onto each unit, for subagents' calls, which carry only their time.
    moves: Vec<(Instant, WorkUnit)>,
    /// A message is written on several lines, each repeating its usage, so it is charged once.
    messages: HashMap<String, Charge>,
    unnamed: Vec<Charge>,
}

/// Tokens charged to a unit, at the time of the line that reported them.
type Charge = (Option<WorkUnit>, Option<Instant>, TokenCount);

impl SessionLedger {
    pub fn new() -> Self {
        Self::default()
    }

    /// A line of the session's own transcript.
    pub fn read_line(&mut self, line: &str) {
        let Some(entry) = TranscriptEntry::parse(line) else { return };
        match entry.kind.as_str() {
            "user" if !entry.is_sidechain => {
                if let Some(unit) = unit_of_prompt(&entry.prompt_text()) {
                    self.move_to(entry.at(), self.continued(unit));
                }
            }
            "assistant" => {
                self.charge(&entry, self.current.clone());
                if entry.is_sidechain {
                    return;
                }
                for (tool, input) in entry.rekall_calls() {
                    if let Some(next) = self.after_call(tool, &input) {
                        self.move_to(entry.at(), next);
                    }
                }
            }
            _ => {}
        }
    }

    /// A line of one of the session's subagents, charged to what the session was on at its time.
    pub fn read_subagent_line(&mut self, line: &str) {
        let Some(entry) = TranscriptEntry::parse(line) else { return };
        if entry.kind == "assistant" {
            let unit = entry.at().and_then(|at| self.unit_at(at));
            self.charge(&entry, unit);
        }
    }

    /// The tokens charged to each unit, one entry per unit and day.
    pub fn spent(self) -> Vec<UnitUsage> {
        let mut spent: Vec<UnitUsage> = Vec::new();
        let charged = self.messages.into_values().chain(self.unnamed);
        for (unit, at, tokens) in charged {
            let Some(unit) = unit else { continue };
            let day = at.map(|at| at.start_of_day());
            match spent.iter_mut().find(|usage| usage.unit == unit && usage.at == day) {
                Some(usage) => usage.tokens += tokens,
                None => spent.push(UnitUsage { unit, at: day, tokens }),
            }
        }
        spent.sort_by(|a, b| (&a.unit.task_label, &a.unit.step, a.at).cmp(&(&b.unit.task_label, &b.unit.step, b.at)));
        spent
    }

    fn charge(&mut self, entry: &TranscriptEntry, unit: Option<WorkUnit>) {
        let Some(message) = &entry.message else { return };
        let Some(usage) = &message.usage else { return };
        let tokens = usage.count();
        let Some(id) = &message.id else {
            self.unnamed.push((unit, entry.at(), tokens));
            return;
        };
        self.messages
            .entry(id.clone())
            .and_modify(|(_, _, known)| *known = largest(*known, tokens))
            .or_insert((unit, entry.at(), tokens));
    }

    /// Where a Rekall tool call moves the session, if anywhere.
    fn after_call(&self, tool: &str, input: &ToolInput) -> Option<WorkUnit> {
        let task = unit_of_anchors(input.anchors.as_deref()?)?;
        match tool {
            "rekall_context" => Some(self.continued(task)),
            "rekall_step" => match input.state.as_deref()?.trim().to_ascii_lowercase().as_str() {
                "running" => Some(task.on_step(input.step.as_ref().map(|step| step.text()).as_deref())),
                "claimed" | "open" => Some(task),
                _ => None,
            },
            _ => None,
        }
    }

    /// A task named again without a step stays on the step the session is already working.
    fn continued(&self, unit: WorkUnit) -> WorkUnit {
        match &self.current {
            Some(current) if unit.step.is_none() && current.same_task(&unit) => current.clone(),
            _ => unit,
        }
    }

    fn move_to(&mut self, at: Option<Instant>, unit: WorkUnit) {
        if let Some(at) = at {
            self.moves.push((at, unit.clone()));
        }
        self.current = Some(unit);
    }

    fn unit_at(&self, at: Instant) -> Option<WorkUnit> {
        self.moves.iter().rev().find(|(moved, _)| !at.is_before(moved)).map(|(_, unit)| unit.clone())
    }
}

/// Lines of one message can differ while it streams; the last carries the final counts.
fn largest(a: TokenCount, b: TokenCount) -> TokenCount {
    TokenCount {
        input: a.input.max(b.input),
        output: a.output.max(b.output),
        cache_creation: a.cache_creation.max(b.cache_creation),
        cache_read: a.cache_read.max(b.cache_read),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/transcript/session_ledger_tests.rs"]
mod tests;
