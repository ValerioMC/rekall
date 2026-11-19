//! A project's task key: two to four capital letters, unique across every company. A task's code is its project's key, a dash and the task's number.

use std::collections::HashSet;

use rekall_kernel::RekallError;

pub struct TaskKey;

impl TaskKey {
    pub const MIN_LENGTH: usize = 2;
    pub const MAX_LENGTH: usize = 4;

    pub fn normalize(raw: &str) -> Result<String, RekallError> {
        let key = raw.trim().to_ascii_uppercase();
        let well_formed = (Self::MIN_LENGTH..=Self::MAX_LENGTH).contains(&key.len())
            && key.chars().all(|c| c.is_ascii_uppercase());
        if !well_formed {
            return Err(RekallError::illegal(format!(
                "'{raw}' is not a task key: {} to {} letters, no digits. A task code is the key, a dash and a number, like RK-12.",
                Self::MIN_LENGTH,
                Self::MAX_LENGTH
            )));
        }
        Ok(key)
    }

    pub fn is_well_formed(key: &str) -> bool {
        Self::normalize(key).is_ok_and(|normalized| normalized == key)
    }

    /// The first key built from the project's label that nobody holds: initials of its words,
    /// then its first letters, then its first letters closed by each other letter in turn.
    pub fn suggest(label: &str, taken: &HashSet<String>) -> String {
        let words: Vec<String> = label
            .split(|c: char| !c.is_ascii_alphabetic())
            .filter(|word| !word.is_empty())
            .map(str::to_ascii_uppercase)
            .collect();
        let letters: String = words.concat();
        let initials: String = words.iter().filter_map(|word| word.chars().next()).take(Self::MAX_LENGTH).collect();
        let padded = format!("{letters}PROJECT");
        let preferred = [initials, padded[..2].to_string(), padded[..3].to_string(), padded[..4].to_string()];
        if let Some(free) = preferred.into_iter().find(|key| key.len() >= Self::MIN_LENGTH && !taken.contains(key)) {
            return free;
        }
        Self::lettered(&padded[..3], &padded[..2], taken)
    }

    fn lettered(long_stem: &str, short_stem: &str, taken: &HashSet<String>) -> String {
        let alphabet = 'A'..='Z';
        let one_more = alphabet.clone().map(|letter| format!("{long_stem}{letter}"));
        let two_more = alphabet.clone().flat_map(|first| alphabet.clone().map(move |second| format!("{short_stem}{first}{second}")));
        one_more.chain(two_more).find(|key| !taken.contains(key)).unwrap_or_else(|| short_stem.to_string())
    }

    pub fn code(key: &str, number: i32) -> String {
        format!("{key}-{number}")
    }
}

#[cfg(test)]
#[path = "../tests/unit/task_key_tests.rs"]
mod tests;
