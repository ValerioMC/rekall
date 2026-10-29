use rekall_console::filesystem::DirectoryListingView;
use serde::Serialize;

use crate::filesystem::DirectoryEntryResponse;
use crate::filesystem::PathSegmentResponse;

/// One folder as the path picker shows it. Every path in it is absolute and already resolved, so
/// the page never joins, splits or normalises a path itself and never has to know the platform's
/// separator. `readable` is false for a folder that exists but refused to be listed (a macOS
/// privacy-protected one, say): the picker still stands in it and says why it is empty, rather
/// than failing the whole request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryListingResponse {
    pub path: String,
    pub parent: Option<String>,
    pub home: String,
    pub segments: Vec<PathSegmentResponse>,
    pub entries: Vec<DirectoryEntryResponse>,
    pub readable: bool,
    pub truncated: bool,
}

impl From<DirectoryListingView> for DirectoryListingResponse {
    fn from(view: DirectoryListingView) -> Self {
        Self {
            path: view.path,
            parent: view.parent,
            home: view.home,
            segments: view.segments.into_iter().map(PathSegmentResponse::from).collect(),
            entries: view.entries.into_iter().map(DirectoryEntryResponse::from).collect(),
            readable: view.readable,
            truncated: view.truncated,
        }
    }
}
