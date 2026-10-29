//! The name a Claude skill is invoked by (`code-review`, `plugin:skill`). Rekall keeps only the
//! name: the skill itself is a file under `~/.claude` that the session resolves.

use rekall_kernel::RekallError;

pub const SKILL_NAME_MAX: usize = 120;

/// A name is letters, digits and `- _ : .`; anything else could not be typed after a slash.
pub fn validate(name: &str) -> Result<(), RekallError> {
    let allowed = |character: char| character.is_alphanumeric() || matches!(character, '-' | '_' | ':' | '.');
    if name.is_empty() || name.chars().count() > SKILL_NAME_MAX || !name.chars().all(allowed) {
        return Err(RekallError::illegal(format!(
            "'{name}' is not a skill name: 1 to {SKILL_NAME_MAX} letters, digits, '-', '_', ':' or '.'."
        )));
    }
    Ok(())
}
