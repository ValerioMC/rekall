use serde::Deserialize;

#[derive(Deserialize)]
pub(in crate::settings) struct CheckQuery {
    pub(in crate::settings) path: Option<String>,
}
