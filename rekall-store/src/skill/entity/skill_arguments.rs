//! The free text a skill is invoked with (`redesign website ui`). Rekall keeps it as typed:
//! each skill defines its own grammar, so only the length is a rule.

use rekall_kernel::RekallError;

use crate::constraints::Violations;

pub const SKILL_ARGUMENTS_MAX: usize = 300;

/// Trims the text and reads a blank one as no arguments.
pub fn normalize(arguments: Option<&str>) -> Option<String> {
    arguments.map(str::trim).filter(|trimmed| !trimmed.is_empty()).map(str::to_string)
}

pub fn validate(arguments: Option<&str>) -> Result<(), RekallError> {
    Violations::new("skill").max_length("arguments", arguments.map(str::trim), SKILL_ARGUMENTS_MAX).finish()
}

#[cfg(test)]
#[path = "../../../tests/unit/skill/entity/skill_arguments_tests.rs"]
mod tests;
