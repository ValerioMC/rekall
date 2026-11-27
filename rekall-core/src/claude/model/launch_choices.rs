//! What a Claude Code session may be launched with. A choice is checked for its shape only, never
//! against the catalog, so a saved one is still read while the catalog is unavailable.

use rekall_kernel::{jstr, RekallError, Result};

/// The family names Claude Code resolves to a version, accepted in any case.
const MODEL_ALIASES: [&str; 4] = ["opus", "sonnet", "haiku", "fable"];
const EFFORT_LEVELS: [&str; 5] = ["low", "medium", "high", "xhigh", "max"];
const ACCOUNT_DEFAULT: &str = "default";
const LONG_CONTEXT_SUFFIX: &str = "[1m]";
const MODEL_MAX_BYTES: usize = 150;

/// The model to launch with, none for the account default. An alias is lowercased; any other
/// well-formed value keeps the provider's spelling.
pub fn claude_model_choice(value: Option<&str>) -> Result<Option<String>> {
    let Some(value) = value else { return Ok(None) };
    let trimmed = jstr::strip(value);
    let lowercase = trimmed.to_lowercase();
    if lowercase == ACCOUNT_DEFAULT {
        return Ok(None);
    }
    if MODEL_ALIASES.contains(&lowercase.as_str()) {
        return Ok(Some(lowercase));
    }
    if is_argument_safe_model(trimmed) {
        return Ok(Some(trimmed.to_owned()));
    }
    Err(refused(value, "model"))
}

/// The effort to launch with, none for the account default.
pub fn claude_effort_choice(value: Option<&str>) -> Result<Option<String>> {
    let Some(value) = value else { return Ok(None) };
    let lowercase = jstr::strip(value).to_lowercase();
    if lowercase == ACCOUNT_DEFAULT {
        return Ok(None);
    }
    if EFFORT_LEVELS.contains(&lowercase.as_str()) {
        return Ok(Some(lowercase));
    }
    Err(refused(value, "effort"))
}

/// A leading letter or digit keeps it from reading as a flag; a long-context suffix may follow.
fn is_argument_safe_model(value: &str) -> bool {
    let name = value.strip_suffix(LONG_CONTEXT_SUFFIX).unwrap_or(value);
    value.len() <= MODEL_MAX_BYTES
        && name.starts_with(|first: char| first.is_ascii_alphanumeric())
        && name.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"-_.:/@".contains(&byte))
}

fn refused(value: &str, what: &str) -> RekallError {
    RekallError::illegal(format!("'{value}' is not a {what} Claude Code accepts."))
}

#[cfg(test)]
#[path = "../../../tests/unit/claude/model/launch_choices_tests.rs"]
mod tests;
