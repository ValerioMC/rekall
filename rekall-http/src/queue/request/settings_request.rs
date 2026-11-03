use rekall_console::queue::RunQueueSettingsCommand;
use rekall_store::SessionKind;
use serde::Deserialize;

/// `ceilingPercent` null is no ceiling; `model` and `effort` blank are the account's, a missing `sessionKind` is `TERMINAL`, a missing `parallelSessions` is one.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::queue) struct SettingsRequest {
    pub ceiling_percent: Option<i32>,
    pub skip_permissions: Option<bool>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub session_kind: Option<SessionKind>,
    pub parallel_sessions: Option<i32>,
}

impl From<SettingsRequest> for RunQueueSettingsCommand {
    fn from(request: SettingsRequest) -> Self {
        Self { ceiling_percent: request.ceiling_percent, skip_permissions: request.skip_permissions == Some(true), model: request.model, effort: request.effort, session_kind: request.session_kind.unwrap_or_default(), parallel_sessions: request.parallel_sessions }
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/queue/request/settings_request_tests.rs"]
mod tests;
