use serde::Deserialize;

/// The term of a catalog search.
#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}
