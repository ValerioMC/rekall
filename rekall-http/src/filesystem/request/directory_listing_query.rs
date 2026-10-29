use serde::Deserialize;

/// Where the path picker lists from: a base folder and a path under it.
#[derive(Deserialize)]
pub struct DirectoryListingQuery {
    pub base: Option<String>,
    pub path: Option<String>,
}
