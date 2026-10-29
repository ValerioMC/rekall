use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct UsageQuery {
    pub(super) refresh: Option<String>,
}
