use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct AddRequest {
    pub(super) path: Option<String>,
    pub(super) label: Option<String>,
}
