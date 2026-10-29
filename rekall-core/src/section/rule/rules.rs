//! What every writer of a section holds to, so a session and the console refuse the same things.

use rekall_kernel::{jstr, Id, RekallError, Result};
use rekall_store::project_section::{self, BODY_MAX_CHARACTERS, TITLE_MAX_CHARACTERS};
use rekall_store::repository as repo;
use sea_orm::ConnectionTrait;

use crate::text_rule;

/// The title as stored: trimmed, present, and within the column.
pub fn title(title: Option<&str>) -> Result<String> {
    let title = text_rule::present(title).ok_or_else(|| RekallError::illegal("A section needs a title."))?;
    text_rule::within(title, TITLE_MAX_CHARACTERS, |length| {
        format!("A section's title is capped at {TITLE_MAX_CHARACTERS} characters and this one is {length}.")
    })
}

/// A body a session sends: present, stripped and within the column. A person may leave one empty.
pub fn written_body(body: Option<&str>) -> Result<String> {
    let body = text_rule::present(body).ok_or_else(|| {
        RekallError::illegal("A section needs a body. An empty one is something a person makes in the console.")
    })?;
    text_rule::within(body, BODY_MAX_CHARACTERS, |length| {
        format!("A section is capped at {BODY_MAX_CHARACTERS} characters and this one is {length}. Split it.")
    })
}

/// The section of `project_id` already titled `wanted` (ignoring case), other than `except`.
pub async fn clashing(
    db: &impl ConnectionTrait,
    project_id: Id,
    wanted: &str,
    except: Option<Id>,
) -> Result<Option<project_section::Model>> {
    let sections = repo::project_section::list_of_project(db, project_id).await?;
    Ok(sections
        .into_iter()
        .find(|section| Some(section.id) != except && jstr::equals_ignore_case(jstr::strip(&section.title), wanted)))
}

pub fn clash_refusal(existing: &project_section::Model) -> RekallError {
    RekallError::illegal(format!(
        "This project already has a section titled '{}'. Nothing was written: pass `replace` to rewrite that \
         section, or pick another title to add a new one.",
        existing.title
    ))
}

#[cfg(test)]
#[path = "../../../tests/unit/section/rule/rules_tests.rs"]
mod tests;
