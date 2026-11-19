//! What every writer of a component holds to, so a session and the console refuse the same things.

use rekall_kernel::{jstr, RekallError, Result};
use rekall_store::project_component::{DESCRIPTION_MAX_CHARACTERS, NAME_MAX_CHARACTERS, SUMMARY_MAX_CHARACTERS};

use crate::text_rule;

/// The folder as stored: stripped, present, and without a trailing separator.
pub fn folder(folder: Option<&str>) -> Result<String> {
    let folder = text_rule::present(folder).ok_or_else(|| RekallError::illegal("A component needs a folder."))?;
    let trimmed = folder.trim_end_matches(['/', '\\']);
    Ok(if trimmed.is_empty() { folder } else { trimmed }.to_string())
}

pub fn name(name: Option<&str>) -> Result<String> {
    let name = text_rule::present(name).ok_or_else(|| RekallError::illegal("A component needs a name."))?;
    text_rule::within(name, NAME_MAX_CHARACTERS, |length| {
        format!("A component's name is capped at {NAME_MAX_CHARACTERS} characters and this one is {length}.")
    })
}

/// What every context carries, so a session's summary has to be there and short.
pub fn written_summary(summary: Option<&str>) -> Result<String> {
    let summary = text_rule::present(summary).ok_or_else(|| RekallError::illegal("A component needs a summary: a few sentences saying what it is for."))?;
    text_rule::within(summary, SUMMARY_MAX_CHARACTERS, |length| {
        format!("A summary is capped at {SUMMARY_MAX_CHARACTERS} characters and this one is {length}. The detail belongs in the description.")
    })
}

pub fn written_description(description: Option<&str>) -> Result<String> {
    let description = text_rule::present(description).ok_or_else(|| RekallError::illegal("A component needs a description."))?;
    text_rule::within(description, DESCRIPTION_MAX_CHARACTERS, |length| {
        format!("A description is capped at {DESCRIPTION_MAX_CHARACTERS} characters and this one is {length}. Shorten it.")
    })
}

pub fn is_same_name(left: &str, right: &str) -> bool {
    jstr::equals_ignore_case(jstr::strip(left), jstr::strip(right))
}

#[cfg(test)]
#[path = "../../../tests/unit/component/rule/rules_tests.rs"]
mod tests;
