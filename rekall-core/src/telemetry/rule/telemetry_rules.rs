use rekall_kernel::{RekallError, Result};
use rekall_store::telemetry_setting::{HOST_MAX_CHARACTERS, PUBLIC_KEY_MAX_CHARACTERS};

use crate::text_rule;

const SECRET_KEY_MAX_CHARACTERS: usize = 255;

/// An http(s) address with no spaces, kept without its trailing slash.
pub fn host(text: Option<&str>) -> Result<String> {
    let given = text_rule::present(text).ok_or_else(|| RekallError::illegal("Give the address of the Langfuse instance."))?;
    if !(given.starts_with("http://") || given.starts_with("https://")) || given.chars().any(char::is_whitespace) {
        return Err(RekallError::illegal("The address starts with http:// or https:// and has no spaces."));
    }
    let kept = text_rule::within(given.trim_end_matches('/'), HOST_MAX_CHARACTERS, |length| {
        format!("The address is at most {HOST_MAX_CHARACTERS} characters and this one is {length}.")
    })?;
    Ok(kept)
}

/// One word, or empty when none is given.
pub fn public_key(text: Option<&str>) -> Result<String> {
    let Some(given) = text_rule::present(text) else { return Ok(String::new()) };
    one_word(given, "public key", PUBLIC_KEY_MAX_CHARACTERS)
}

/// One word, or `None` when none is given, which keeps the stored one.
pub fn secret_key(text: Option<&str>) -> Result<Option<String>> {
    text_rule::present(text).map(|given| one_word(given, "secret key", SECRET_KEY_MAX_CHARACTERS)).transpose()
}

fn one_word(given: &str, name: &str, max: usize) -> Result<String> {
    if given.chars().any(char::is_whitespace) {
        return Err(RekallError::illegal(format!("A {name} has no spaces in it. Paste it alone.")));
    }
    text_rule::within(given, max, |length| format!("A {name} is at most {max} characters and this one is {length}."))
}

#[cfg(test)]
#[path = "../../../tests/unit/telemetry/rule/telemetry_rules_tests.rs"]
mod tests;
