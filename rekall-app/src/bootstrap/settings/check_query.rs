use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct CheckQuery {
    pub(super) path: Option<String>,
}
