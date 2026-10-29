use serde::Deserialize;

#[derive(Deserialize)]
pub(in crate::claude) struct ResourceQuery {
    pub(in crate::claude) path: Option<String>,
}
