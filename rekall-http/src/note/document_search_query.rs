use serde::Deserialize;

/// The text a note search looks for.
#[derive(Deserialize)]
pub struct DocumentSearchQuery {
    pub query: Option<String>,
}
