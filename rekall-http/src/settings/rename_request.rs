use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct RenameRequest {
    pub(super) label: Option<String>,
}
