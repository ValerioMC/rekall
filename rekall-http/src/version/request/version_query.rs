use serde::Deserialize;

#[derive(Deserialize)]
pub(in crate::version) struct VersionQuery {
    /// Ask the feed again instead of answering with what is remembered.
    #[serde(default)]
    pub(in crate::version) refresh: bool,
}
