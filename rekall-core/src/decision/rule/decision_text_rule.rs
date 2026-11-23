use rekall_kernel::{RekallError, Result};

use crate::text_rule;

/// The text a decision needs in `field`, stripped and within `max` characters; `hint` says what to write when it is missing.
pub fn required(text: Option<&str>, field: &str, max: usize, hint: &str) -> Result<String> {
    let present = text_rule::present(text).ok_or_else(|| RekallError::illegal(format!("A decision needs {field}: {hint}. Nothing was written.")))?;
    capped(present, field, max)
}

/// The text in `field` when one was sent, stripped and within `max` characters.
pub fn optional(text: Option<&str>, field: &str, max: usize) -> Result<Option<String>> {
    text_rule::present(text).map(|present| capped(present, field, max)).transpose()
}

fn capped(text: &str, field: &str, max: usize) -> Result<String> {
    text_rule::within(text, max, |length| format!("A decision's {field} is capped at {max} characters and this one is {length}. Shorten it."))
}
