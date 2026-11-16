use rekall_console::connection::{ConfluenceImportView, ConfluenceSectionView};
use rekall_console::section::SectionWriteOutcome;
use serde::Serialize;

/// What filling a project from a Confluence space did.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceImportResponse {
    pub space_key: String,
    pub space_name: String,
    pub pages_read: usize,
    pub description_written: bool,
    pub blueprint_written: bool,
    pub sections: Vec<ConfluenceSectionResponse>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceSectionResponse {
    pub title: String,
    pub pages: usize,
    pub outcome: &'static str,
}

impl From<ConfluenceImportView> for ConfluenceImportResponse {
    fn from(view: ConfluenceImportView) -> Self {
        Self {
            space_key: view.space_key,
            space_name: view.space_name,
            pages_read: view.pages_read,
            description_written: view.description_written,
            blueprint_written: view.blueprint_written,
            sections: view.sections.into_iter().map(ConfluenceSectionResponse::from).collect(),
        }
    }
}

impl From<ConfluenceSectionView> for ConfluenceSectionResponse {
    fn from(view: ConfluenceSectionView) -> Self {
        let outcome = match view.outcome {
            SectionWriteOutcome::Created => "created",
            SectionWriteOutcome::Replaced => "replaced",
            SectionWriteOutcome::Kept => "kept",
        };
        Self { title: view.title, pages: view.pages, outcome }
    }
}
