//! What every writer of a section holds to, so a session and the console refuse the same things.

use rekall_common::{jstr, Id, RekallError, Result};
use rekall_model::project_section::{self, BODY_MAX_CHARACTERS, TITLE_MAX_CHARACTERS};
use rekall_repository::repository as repo;
use sea_orm::ConnectionTrait;

/// The title as stored: trimmed, present, and within the column.
pub fn title(title: Option<&str>) -> Result<String> {
    let Some(title) = title.filter(|t| !jstr::is_blank(t)) else {
        return Err(RekallError::illegal("A section needs a title."));
    };
    let text = jstr::strip(title);
    let length = jstr::len(text);
    if length > TITLE_MAX_CHARACTERS {
        return Err(RekallError::illegal(format!(
            "A section's title is capped at {TITLE_MAX_CHARACTERS} characters and this one is {length}."
        )));
    }
    Ok(text.to_string())
}

/// A body a session sends: present, stripped and within the column. A person may leave one empty.
pub fn written_body(body: Option<&str>) -> Result<String> {
    let Some(body) = body.filter(|b| !jstr::is_blank(b)) else {
        return Err(RekallError::illegal("A section needs a body. An empty one is something a person makes in the console."));
    };
    let text = jstr::strip(body);
    let length = jstr::len(text);
    if length > BODY_MAX_CHARACTERS {
        return Err(RekallError::illegal(format!(
            "A section is capped at {BODY_MAX_CHARACTERS} characters and this one is {length}. Split it."
        )));
    }
    Ok(text.to_string())
}

/// The section of `project_id` already titled `wanted` (ignoring case), other than `except`.
pub async fn clashing(
    db: &impl ConnectionTrait,
    project_id: Id,
    wanted: &str,
    except: Option<Id>,
) -> Result<Option<project_section::Model>> {
    let sections = repo::project_section::find_by_project_id_order_by_position_asc(db, project_id).await?;
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
#[path = "../../tests/unit/section/rules_tests.rs"]
mod tests;
