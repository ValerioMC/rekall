use crate::section::SectionWriteOutcome;

/// What filling a project from a Confluence space did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfluenceImportView {
    pub space_key: String,
    pub space_name: String,
    pub pages_read: usize,
    pub description_written: bool,
    pub blueprint_written: bool,
    pub sections: Vec<ConfluenceSectionView>,
}

/// One section the space produced and what became of it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfluenceSectionView {
    pub title: String,
    pub pages: usize,
    pub outcome: SectionWriteOutcome,
}
