/// A check to create or edit. On an edit, a field left out keeps what the check has and a blank
/// folder clears it; on a create, `label`, `command` and `timeout_seconds` are required.
#[derive(Clone, Debug, Default)]
pub struct CheckCommand {
    pub label: Option<String>,
    pub command: Option<String>,
    pub folder: Option<String>,
    pub timeout_seconds: Option<i32>,
}
