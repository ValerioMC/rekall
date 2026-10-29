use rekall_console::settings::AddDatabaseCommand;
use serde::Deserialize;

#[derive(Deserialize)]
pub(in crate::settings) struct AddRequest {
    pub(in crate::settings) path: Option<String>,
    pub(in crate::settings) label: Option<String>,
}

impl From<AddRequest> for AddDatabaseCommand {
    fn from(request: AddRequest) -> Self {
        Self { path: request.path, label: request.label }
    }
}
