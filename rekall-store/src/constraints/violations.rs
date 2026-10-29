use rekall_kernel::{jstr, RekallError};

use crate::slug::Slug;

/// Collects every rule a row breaks before it is written, and reports them together as one
/// `IllegalArgument`: a value the caller sent that the model cannot hold.
pub struct Violations {
    entity: &'static str,
    broken: Vec<String>,
}

impl Violations {
    pub fn new(entity: &'static str) -> Self {
        Self { entity, broken: Vec::new() }
    }

    /// Absent, empty or whitespace only is refused.
    pub fn not_blank(&mut self, property: &str, value: Option<&str>) -> &mut Self {
        if jstr::is_null_or_blank(value) {
            self.broken.push(format!("{property} must not be blank"));
        }
        self
    }

    /// Absent is allowed; the length is counted the way every cap in Rekall is (see `jstr::len`).
    pub fn max_length(&mut self, property: &str, value: Option<&str>, max: usize) -> &mut Self {
        if let Some(length) = value.map(jstr::len).filter(|length| *length > max) {
            self.broken.push(format!("{property} is capped at {max} characters and this one is {length}"));
        }
        self
    }

    pub fn slug(&mut self, property: &str, value: Option<&str>) -> &mut Self {
        if value.is_some_and(|v| !Slug::matches(v)) {
            self.broken.push(format!("{property} must be a slug: lowercase letters, digits, '-', '_' or '.'"));
        }
        self
    }

    pub fn finish(&self) -> Result<(), RekallError> {
        if self.broken.is_empty() {
            return Ok(());
        }
        Err(RekallError::illegal(format!("{}: {}.", self.entity, self.broken.join("; "))))
    }
}

#[cfg(test)]
#[path = "../../tests/unit/constraints/violations_tests.rs"]
mod tests;
