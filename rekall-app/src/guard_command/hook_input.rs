use rekall_core::guard::GuardEvent;
use serde::Deserialize;

use super::guard_mode::GuardMode;

/// The JSON Claude Code writes to a hook's standard input; only the fields the guard reads.
#[derive(Debug, Deserialize)]
pub(super) struct HookInput {
    pub(super) cwd: String,
    tool_name: Option<String>,
    tool_input: Option<ToolInput>,
    /// Set on a `Stop` that follows a block by a stop hook.
    #[serde(default)]
    pub(super) stop_hook_active: bool,
}

#[derive(Debug, Deserialize)]
struct ToolInput {
    command: Option<String>,
    file_path: Option<String>,
    path: Option<String>,
}

impl HookInput {
    pub(super) fn event(&self, mode: GuardMode) -> GuardEvent {
        match mode {
            GuardMode::BeforeStop => GuardEvent::BeforeStop,
            GuardMode::BeforeTool => GuardEvent::BeforeTool {
                tool_name: self.tool_name.clone().unwrap_or_default(),
                command_or_path: self.tool_input.as_ref().and_then(ToolInput::command_or_path).unwrap_or_default(),
            },
        }
    }
}

impl ToolInput {
    fn command_or_path(&self) -> Option<String> {
        self.command.clone().or_else(|| self.file_path.clone()).or_else(|| self.path.clone())
    }
}
