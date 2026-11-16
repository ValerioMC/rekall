//! What turns a project's Confluence binding into a space to read: the binding's target is the
//! space key.

use rekall_kernel::{RekallError, Result};

/// The most pages one import reads from a space.
pub(in crate::connection) const PAGE_LIMIT: u16 = 200;

/// The most targets one suggestion list offers.
pub(in crate::connection) const SUGGESTION_LIMIT: u16 = 25;

/// The binding's target as a space key: letters, digits, `_` and `-`, or `~` and a name for a personal space.
pub(in crate::connection) fn space_key_of(target: Option<&str>) -> Result<String> {
    let key = target.map(str::trim).unwrap_or_default();
    let name = key.strip_prefix('~').unwrap_or(key);
    let valid = !name.is_empty() && name.chars().all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.' | '@'));
    if !valid {
        return Err(RekallError::illegal("Set this project's target for the Confluence connection to a Confluence space key, such as ENG."));
    }
    Ok(key.to_string())
}

#[cfg(test)]
#[path = "../../../tests/unit/connection/rule/confluence_rules_tests.rs"]
mod tests;
