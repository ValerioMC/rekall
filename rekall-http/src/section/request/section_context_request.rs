use rekall_console::section::SectionContextCommand;
use serde::Deserialize;

/// A section context to create, or to rename.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SectionContextRequest {
    pub title: Option<String>,
}

impl From<SectionContextRequest> for SectionContextCommand {
    fn from(request: SectionContextRequest) -> Self {
        Self { title: request.title }
    }
}
