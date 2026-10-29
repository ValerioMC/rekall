use rekall_console::settings::AddDatabaseCommand;
use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct AddRequest {
    pub(super) path: Option<String>,
    pub(super) label: Option<String>,
}

impl From<AddRequest> for AddDatabaseCommand {
    fn from(request: AddRequest) -> Self {
        Self { path: request.path, label: request.label }
    }
}
