use super::unified_diff::{DiffFileSpan, UnifiedDiff};
use crate::commit::view::{ChangedFileMetadata, DiffHunkView};

#[derive(Default)]
pub(in crate::commit) struct IndexedPatch {
    patch: String,
    files: Vec<DiffFileSpan>,
}

impl IndexedPatch {
    pub fn read(patch: String) -> Option<Self> {
        let files = UnifiedDiff::index(&patch);
        (!files.is_empty()).then_some(Self { patch, files })
    }

    pub fn metadata(&self) -> Vec<ChangedFileMetadata> {
        self.files.iter().map(|file| file.metadata.clone()).collect()
    }

    pub fn hunks(&self, path: &str) -> Option<Vec<DiffHunkView>> {
        let mut matches = self.files.iter().filter(|file| file.metadata.path == path).peekable();
        matches.peek()?;
        Some(
            matches
                .flat_map(|file| UnifiedDiff::parse(&self.patch[file.bytes.clone()]))
                .flat_map(|file| file.hunks)
                .collect(),
        )
    }
}
