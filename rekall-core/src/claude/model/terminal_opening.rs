use rekall_kernel::{Id, RekallError, Result};

use crate::claude::{SessionKind, SessionProvider, TerminalMode, TerminalTarget, TerminalView};

/// What a terminal is opened on and how: the step it follows, whether it skips permission
/// prompts, the model and effort asked for, the work its first line starts, and the kind of
/// session it runs in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TerminalOpening {
    pub target: TerminalTarget,
    pub step_id: Option<Id>,
    pub provider: SessionProvider,
    pub skip_permissions: bool,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub mode: TerminalMode,
    pub kind: SessionKind,
}

impl TerminalOpening {
    pub fn kind_for_live_session(&self, existing: &TerminalView) -> Result<SessionKind> {
        let requires_kind = matches!(self.mode, TerminalMode::Work | TerminalMode::Merge | TerminalMode::MergePullRequest { .. });
        if existing.provider == self.provider && (!requires_kind || existing.kind == self.kind) {
            return Ok(existing.kind);
        }
        Err(RekallError::conflict(format!(
            "{} {} is already open here. Close it to continue with {} {}.",
            existing.provider.label(),
            kind_label(existing.kind),
            self.provider.label(),
            kind_label(self.kind),
        )))
    }
}

fn kind_label(kind: SessionKind) -> &'static str {
    match kind {
        SessionKind::Chat => "Chat",
        SessionKind::Terminal => "Terminal",
    }
}
