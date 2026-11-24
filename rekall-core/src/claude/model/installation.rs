use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Installation {
    pub status: &'static str,
    pub endpoint: String,
    pub registered_url: Option<String>,
    pub folder_scoped: Vec<String>,
    pub command_installed: bool,
    pub outdated_files: Vec<String>,
    pub cli_path: Option<String>,
    pub manual_command: String,
}

impl Installation {
    pub const OUTDATED: &'static str = "OUTDATED";

    pub fn needs_command_update(&self) -> bool {
        self.status == Self::OUTDATED && !self.outdated_files.is_empty()
    }
}
