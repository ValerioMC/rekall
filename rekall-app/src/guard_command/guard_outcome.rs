pub const EXIT_ALLOW: u8 = 0;
pub const EXIT_BLOCK: u8 = 2;

/// `reason` goes to standard error, where Claude Code reads it when the exit code is 2.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuardOutcome {
    pub exit_code: u8,
    pub reason: Option<String>,
}

impl GuardOutcome {
    pub fn allow() -> Self {
        Self { exit_code: EXIT_ALLOW, reason: None }
    }

    pub fn block(reason: String) -> Self {
        Self { exit_code: EXIT_BLOCK, reason: Some(reason) }
    }
}
