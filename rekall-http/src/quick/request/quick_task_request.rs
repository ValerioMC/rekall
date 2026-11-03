use rekall_console::quick::QuickTaskCommand;
use rekall_kernel::Id;
use serde::Deserialize;

/// What to do, in the person's words, and how the session that does it is opened. `model` and
/// `effort` left out leave the account default.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct QuickTaskRequest {
    pub text: Option<String>,
    pub skip_permissions: Option<bool>,
    pub model: Option<String>,
    pub effort: Option<String>,
}

impl QuickTaskRequest {
    pub fn command(self, project_id: Id) -> QuickTaskCommand {
        QuickTaskCommand {
            project_id,
            text: self.text.unwrap_or_default(),
            skip_permissions: self.skip_permissions.unwrap_or(false),
            model: self.model,
            effort: self.effort,
        }
    }
}
