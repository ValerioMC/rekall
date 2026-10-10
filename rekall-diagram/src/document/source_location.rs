use serde::{Deserialize, Serialize};

/// A span of real code an element is implemented by. `file` is relative to the project's
/// folder; lines are one-based and inclusive, and absent when the whole file is meant.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceLocation {
    pub file: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_line: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_line: Option<u32>,
    /// The function, method or type the span sits in, when it has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
}

impl SourceLocation {
    /// Forward slashes, no leading `./`: how a path is compared, never how it is stored.
    pub fn normalise(path: &str) -> String {
        let unified = path.trim().replace('\\', "/");
        unified.trim_start_matches("./").to_string()
    }
}
