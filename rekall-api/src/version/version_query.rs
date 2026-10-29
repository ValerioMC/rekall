use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct VersionQuery {
    /// Ask the feed again instead of answering with what is remembered.
    #[serde(default)]
    pub(super) refresh: bool,
}
