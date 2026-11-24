use serde::Deserialize;

/// One hunk of the diff an edit made: its lines, each prefixed by a space, `+` or `-`.
#[derive(Debug, Deserialize)]
pub(in crate::chat) struct WirePatchHunk {
    pub(in crate::chat) lines: Vec<String>,
}
