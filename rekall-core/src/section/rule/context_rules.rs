//! What every writer of a section context holds to, and the context a new section lands in when
//! its writer names none, so a session and the console place a page the same way.

use rekall_kernel::{jstr, Id, Instant, RekallError, Result};
use rekall_store::repository as repo;
use rekall_store::section_context::{self, TITLE_MAX_CHARACTERS};
use sea_orm::{ActiveModelTrait, ConnectionTrait, IntoActiveModel};

use crate::text_rule;

pub const GENERAL_TITLE: &str = "General";

/// The title as stored: trimmed, present, and within the column.
pub fn title(title: Option<&str>) -> Result<String> {
    let title = text_rule::present(title).ok_or_else(|| RekallError::illegal("A context needs a title."))?;
    text_rule::within(title, TITLE_MAX_CHARACTERS, |length| {
        format!("A context's title is capped at {TITLE_MAX_CHARACTERS} characters and this one is {length}.")
    })
}

/// The context of `project_id` already titled `wanted` (ignoring case), other than `except`.
pub async fn clashing(
    db: &impl ConnectionTrait,
    project_id: Id,
    wanted: &str,
    except: Option<Id>,
) -> Result<Option<section_context::Model>> {
    let contexts = repo::section_context::list_of_project(db, project_id).await?;
    Ok(contexts
        .into_iter()
        .find(|context| Some(context.id) != except && jstr::equals_ignore_case(jstr::strip(&context.title), wanted)))
}

/// A context added after the project's last one.
pub async fn add(db: &impl ConnectionTrait, project_id: Id, title: String, now: Instant) -> Result<section_context::Model> {
    let existing = repo::section_context::list_of_project(db, project_id).await?;
    let context = section_context::Model {
        id: Id::random(),
        project_id,
        title,
        position: existing.iter().map(|context| context.position + 1).max().unwrap_or(0),
        created_at: now,
        updated_at: now,
    };
    context.validate()?;
    context.clone().into_active_model().insert(db).await?;
    Ok(context)
}

/// Where a new section goes: the context `named` (added when the project has none by that title),
/// or the project's first, which is "General" on a project with no context yet.
pub async fn landing(db: &impl ConnectionTrait, project_id: Id, named: Option<&str>, now: Instant) -> Result<section_context::Model> {
    let contexts = repo::section_context::list_of_project(db, project_id).await?;
    let Some(wanted) = text_rule::present(named) else {
        return match contexts.into_iter().next() {
            Some(first) => Ok(first),
            None => add(db, project_id, GENERAL_TITLE.to_string(), now).await,
        };
    };
    match contexts.into_iter().find(|context| jstr::equals_ignore_case(jstr::strip(&context.title), wanted)) {
        Some(found) => Ok(found),
        None => add(db, project_id, title(Some(wanted))?, now).await,
    }
}

/// The place a section takes at the end of `context_id`.
pub async fn next_section_position(db: &impl ConnectionTrait, context_id: Id) -> Result<i32> {
    let held = repo::project_section::list_of_context(db, context_id).await?;
    Ok(held.iter().map(|section| section.position + 1).max().unwrap_or(0))
}

#[cfg(test)]
#[path = "../../../tests/unit/section/rule/context_rules_tests.rs"]
mod tests;
