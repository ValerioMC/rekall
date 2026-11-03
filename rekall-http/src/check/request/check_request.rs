use rekall_console::check::CheckCommand;
use serde::Deserialize;

/// A check to create or edit. On an edit, a field left out keeps what the check has and a blank
/// folder clears it; on a create, `label`, `command` and `timeoutSeconds` are required.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CheckRequest {
    pub label: Option<String>,
    pub command: Option<String>,
    pub folder: Option<String>,
    pub timeout_seconds: Option<i32>,
}

impl From<CheckRequest> for CheckCommand {
    fn from(request: CheckRequest) -> Self {
        Self {
            label: request.label,
            command: request.command,
            folder: request.folder,
            timeout_seconds: request.timeout_seconds,
        }
    }
}
