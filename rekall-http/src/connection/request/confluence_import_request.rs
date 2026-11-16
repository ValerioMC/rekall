use rekall_console::connection::ConfluenceImportCommand;
use serde::Deserialize;

/// Whether the import rewrites the texts and sections the project already has.
#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ConfluenceImportRequest {
    pub replace: bool,
}

impl From<ConfluenceImportRequest> for ConfluenceImportCommand {
    fn from(request: ConfluenceImportRequest) -> Self {
        Self { replace: request.replace }
    }
}
