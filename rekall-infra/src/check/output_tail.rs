const MAX_LINES: usize = 60;
const MAX_CHARACTERS: usize = 4_000;

/// The last lines a command printed, as text: undecodable bytes become U+FFFD, and what exceeds the
/// line or character cap is cut from the start.
pub fn output_tail(output: &[u8]) -> String {
    let text = String::from_utf8_lossy(output);
    let lines: Vec<&str> = text.lines().collect();
    let kept = lines[lines.len().saturating_sub(MAX_LINES)..].join("\n");
    let excess = kept.chars().count().saturating_sub(MAX_CHARACTERS);
    kept.chars().skip(excess).collect()
}

#[cfg(test)]
#[path = "../../tests/unit/check/output_tail_tests.rs"]
mod tests;
