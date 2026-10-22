//! What turns a project's Jira binding into a search, and a Jira issue into the task it is imported
//! as. An issue is imported once: its task is labelled with the issue key.

use rekall_common::{Id, RekallError, Result};
use rekall_model::Slug;
use rekall_service::connection::JiraIssue;

use crate::catalog::TaskRequest;

/// The most issues one listing shows and one import takes.
pub(super) const ISSUE_LIMIT: u16 = 50;

const TITLE_MAX_CHARACTERS: usize = 200;
const DESCRIPTION_MAX_CHARACTERS: usize = 100_000;

/// The binding's target as a Jira project key, upper-cased as Jira writes it.
pub(super) fn project_key_of(target: Option<&str>) -> Result<String> {
    let key = target.map(str::trim).unwrap_or_default().to_uppercase();
    if !is_project_key(&key) {
        return Err(RekallError::illegal("Set this project's target for the Jira connection to a Jira project key, such as ENG."));
    }
    Ok(key)
}

/// The project's issues not yet done, the most recently updated first.
pub(super) fn open_issues_query(project_key: &str) -> String {
    format!("project = \"{project_key}\" AND statusCategory != Done ORDER BY updated DESC")
}

/// The issues named by `keys`, each of which has to belong to the bound project.
pub(super) fn chosen_issues_query(project_key: &str, keys: &[String]) -> Result<String> {
    if keys.is_empty() {
        return Err(RekallError::illegal("Choose at least one Jira issue to import."));
    }
    if keys.len() > usize::from(ISSUE_LIMIT) {
        return Err(RekallError::illegal(format!("Import at most {ISSUE_LIMIT} Jira issues at a time.")));
    }
    let mut chosen = Vec::with_capacity(keys.len());
    for key in keys {
        chosen.push(issue_key_of(project_key, key)?);
    }
    chosen.sort();
    chosen.dedup();
    Ok(format!("key in ({}) ORDER BY key ASC", chosen.join(", ")))
}

pub(super) fn issue_url(site: &str, key: &str) -> String {
    format!("{}/browse/{key}", site.trim_end_matches('/'))
}

/// The label an issue's task carries, which is also how an imported issue is recognised.
pub(super) fn task_label(key: &str) -> String {
    key.to_lowercase()
}

pub(super) fn task_request(project_id: Id, site: &str, issue: &JiraIssue) -> TaskRequest {
    TaskRequest {
        label: Some(task_label(&issue.key)),
        title: Some(task_title(issue)),
        description: Some(task_description(site, issue)),
        project_id: Some(project_id),
        ..TaskRequest::default()
    }
}

fn task_title(issue: &JiraIssue) -> String {
    let summary = issue.summary.trim();
    let title = if summary.is_empty() { issue.key.as_str() } else { summary };
    title.chars().take(TITLE_MAX_CHARACTERS).collect()
}

fn task_description(site: &str, issue: &JiraIssue) -> String {
    let origin = format!("Imported from Jira: [{}]({}).", issue.key, issue_url(site, &issue.key));
    let text = match issue.description.as_deref().map(str::trim).filter(|text| !text.is_empty()) {
        Some(description) => format!("{origin}\n\n{description}"),
        None => origin,
    };
    text.chars().take(DESCRIPTION_MAX_CHARACTERS).collect()
}

/// `raw` upper-cased, when it is an issue of `project_key` such as `ENG-12`.
fn issue_key_of(project_key: &str, raw: &str) -> Result<String> {
    let key = raw.trim().to_uppercase();
    let number = key.strip_prefix(project_key).and_then(|rest| rest.strip_prefix('-'));
    match number {
        Some(number) if !number.is_empty() && number.bytes().all(|byte| byte.is_ascii_digit()) => Ok(key),
        _ => Err(RekallError::illegal(format!("'{raw}' is not an issue of the Jira project {project_key}."))),
    }
}

/// A letter, then letters, digits or `_`: what Jira accepts as a project key.
fn is_project_key(key: &str) -> bool {
    let mut characters = key.chars();
    characters.next().is_some_and(|first| first.is_ascii_uppercase())
        && characters.all(|character| character.is_ascii_uppercase() || character.is_ascii_digit() || character == '_')
        && Slug::matches(&task_label(key))
}

#[cfg(test)]
#[path = "../../tests/unit/connection/jira_rules_tests.rs"]
mod tests;
