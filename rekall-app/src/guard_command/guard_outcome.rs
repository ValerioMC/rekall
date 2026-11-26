use rekall_core::claude::SessionProvider;
use rekall_core::guard::GuardVerdict;
use serde_json::json;

use super::guard_mode::GuardMode;

pub const EXIT_ALLOW: u8 = 0;
const EXIT_BLOCK: u8 = 2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuardOutcome {
    pub exit_code: u8,
    pub stderr: Option<String>,
    pub stdout: Option<String>,
}

impl GuardOutcome {
    pub fn allow() -> Self {
        Self { exit_code: EXIT_ALLOW, stderr: None, stdout: None }
    }

    pub fn of(provider: SessionProvider, mode: GuardMode, verdict: GuardVerdict) -> Self {
        let GuardVerdict::Block { reason } = verdict else {
            return Self::allow();
        };
        if provider == SessionProvider::Claude {
            return Self { exit_code: EXIT_BLOCK, stderr: Some(reason), stdout: None };
        }
        let output = match mode {
            GuardMode::BeforeTool => json!({
                "hookSpecificOutput": {
                    "hookEventName": "PreToolUse",
                    "permissionDecision": "deny",
                    "permissionDecisionReason": reason
                }
            }),
            GuardMode::BeforeStop => json!({ "decision": "block", "reason": reason }),
        };
        Self { exit_code: EXIT_ALLOW, stderr: None, stdout: Some(output.to_string()) }
    }
}
