use serde::Deserialize;

#[derive(Deserialize)]
pub(in crate::settings) struct RenameRequest {
    pub(in crate::settings) label: Option<String>,
}
