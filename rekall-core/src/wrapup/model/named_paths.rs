use std::sync::LazyLock;

use regex::Regex;

pub const MAX_NAMED_PATHS: usize = 20;

const KNOWN_EXTENSIONS: [&str; 15] =
    ["rs", "ts", "tsx", "vue", "js", "md", "toml", "sql", "json", "yml", "yaml", "css", "html", "sh", "lock"];

/// The files and folders a wrapup names between backticks. A heuristic: text that is a path in
/// plain characters and either holds a `/` and ends in a known extension or starts with `rekall-`.
/// A path written without backticks is not seen.
pub struct NamedPaths;

impl NamedPaths {
    pub fn in_text(markdown: &str) -> Vec<String> {
        static SPAN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"`([^`\n]+)`").expect("a valid pattern"));
        let mut found: Vec<String> = Vec::new();
        for span in SPAN.captures_iter(markdown) {
            let candidate = span[1].trim();
            if looks_like_path(candidate) && !found.iter().any(|known| known == candidate) {
                found.push(candidate.to_string());
            }
        }
        found
    }
}

fn looks_like_path(candidate: &str) -> bool {
    static PLAIN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z0-9._/-]+$").expect("a valid pattern"));
    if !PLAIN.is_match(candidate) || candidate.starts_with('/') || candidate.contains("..") {
        return false;
    }
    candidate.starts_with("rekall-") || (candidate.contains('/') && has_known_extension(candidate))
}

fn has_known_extension(candidate: &str) -> bool {
    let name = candidate.rsplit('/').next().unwrap_or(candidate);
    name.rsplit_once('.').is_some_and(|(_, extension)| KNOWN_EXTENSIONS.contains(&extension))
}

#[cfg(test)]
#[path = "../../../tests/unit/wrapup/model/named_paths_tests.rs"]
mod tests;
