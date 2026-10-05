//! The rule every free text a writer sends is held to: blank reads as absent, and what is kept is
//! stripped and measured in UTF-16 units, the way the console counts. Each caller words its own
//! refusals, since a session reads them as instructions.

use rekall_common::{jstr, RekallError, Result};

/// The text stripped, or `None` when it is absent or blank.
pub fn present(text: Option<&str>) -> Option<&str> {
    text.filter(|t| !jstr::is_blank(t)).map(jstr::strip)
}

/// `text` when it fits in `max` characters; otherwise `too_long`, given the length found, words the 400.
pub fn within(text: &str, max: usize, too_long: impl FnOnce(usize) -> String) -> Result<String> {
    let length = jstr::len(text);
    if length > max {
        return Err(RekallError::illegal(too_long(length)));
    }
    Ok(text.to_string())
}

#[cfg(test)]
#[path = "../tests/unit/text_rule_tests.rs"]
mod tests;
