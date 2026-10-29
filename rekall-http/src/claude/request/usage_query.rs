use serde::Deserialize;

#[derive(Deserialize)]
pub(in crate::claude) struct UsageQuery {
    pub(in crate::claude) refresh: Option<String>,
}
