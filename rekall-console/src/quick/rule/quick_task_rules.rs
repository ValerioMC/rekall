use rekall_kernel::{jstr, Id};
use rekall_store::Slug;

use crate::catalog::TaskCommand;

const TITLE_MAX_CHARACTERS: usize = 80;
const FALLBACK_LABEL: &str = "quick-task";

/// The first line of the text, cut at 80 characters.
pub fn title(text: &str) -> String {
    let first_line = jstr::lines(text).into_iter().map(jstr::trim).find(|line| !line.is_empty()).unwrap_or_default();
    first_line.chars().take(TITLE_MAX_CHARACTERS).collect::<String>().trim_end().to_string()
}

/// The title as a label, numbered `-2`, `-3`… after it when the project already uses that label.
pub fn label(title: &str, attempt: usize) -> String {
    let base = Slug::of(Some(title)).unwrap_or_else(|_| FALLBACK_LABEL.to_string());
    if attempt <= 1 {
        return base;
    }
    format!("{base}-{attempt}")
}

pub fn task_request(project_id: Id, text: &str, label: String) -> TaskCommand {
    TaskCommand {
        label: Some(label),
        title: Some(title(text)),
        description: Some(jstr::trim(text).to_string()),
        project_id: Some(project_id),
        quick: true,
        ..TaskCommand::default()
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/quick/rule/quick_task_rules_tests.rs"]
mod tests;
