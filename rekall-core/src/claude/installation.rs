use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Installation {
    pub status: &'static str,
    pub endpoint: String,
    pub registered_url: Option<String>,
    pub folder_scoped: Vec<String>,
    pub command_installed: bool,
    pub cli_path: Option<String>,
    pub manual_command: String,
}
