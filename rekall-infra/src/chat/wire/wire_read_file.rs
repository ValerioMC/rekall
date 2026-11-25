use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::chat) struct WireReadFile {
    pub(in crate::chat) num_lines: u64,
}
