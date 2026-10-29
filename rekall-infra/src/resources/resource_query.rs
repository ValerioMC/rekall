use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct ResourceQuery {
    pub(super) path: Option<String>,
}
