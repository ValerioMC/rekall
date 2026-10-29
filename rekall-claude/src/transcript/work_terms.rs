//! The task and step a session names, read from the two places it names them: the arguments of
//! an `/rk` line, and the `anchors` of a Rekall tool call.

use std::sync::LazyLock;

use regex::Regex;
use rekall_core::token::WorkUnit;

/// The unit an `/rk` prompt is about, when the prompt is one and names a task. Text in double
/// quotes is a request or a directive, never a term.
pub(super) fn unit_of_prompt(text: &str) -> Option<WorkUnit> {
    let terms = Terms::of(slash_arguments(text)?);
    Some(WorkUnit::task(terms.project.as_deref(), terms.single_task()?).on_step(terms.step.as_deref()))
}

/// The task a tool call's `anchors` names, when it names exactly one.
pub(super) fn unit_of_anchors(anchors: &str) -> Option<WorkUnit> {
    let terms = Terms::of(anchors);
    Some(WorkUnit::task(terms.project.as_deref(), terms.single_task()?))
}

/// The arguments of `/rk`, in the form Claude Code records a slash command or as typed.
fn slash_arguments(text: &str) -> Option<&str> {
    static RECORDED: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?s)<command-name>/rk</command-name>.*?<command-args>(.*?)</command-args>").expect("a valid pattern")
    });
    if let Some(found) = RECORDED.captures(text).and_then(|captures| captures.get(1)) {
        return Some(found.as_str());
    }
    text.trim_start().strip_prefix("/rk ")
}

#[derive(Default)]
struct Terms {
    project: Option<String>,
    tasks: Vec<String>,
    step: Option<String>,
}

impl Terms {
    fn of(text: &str) -> Self {
        static QUOTED: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#""[^"]*""#).expect("a valid pattern"));
        let mut terms = Self::default();
        for word in QUOTED.replace_all(text, " ").split_whitespace() {
            terms.take(word);
        }
        terms
    }

    fn take(&mut self, word: &str) {
        let Some((entity, value)) = word.split_once(':') else { return };
        if value.is_empty() {
            return;
        }
        match entity.to_ascii_lowercase().as_str() {
            "project" => self.project = Some(value.to_string()),
            "task" => self.tasks.push(value.to_string()),
            "step" => self.step = Some(value.to_string()),
            _ => {}
        }
    }

    fn single_task(&self) -> Option<&str> {
        match self.tasks.as_slice() {
            [task] => Some(task),
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/transcript/work_terms_tests.rs"]
mod tests;
