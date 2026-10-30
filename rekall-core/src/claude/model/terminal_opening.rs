use rekall_kernel::Id;

use crate::claude::{SessionKind, TerminalMode, TerminalTarget};

/// What a terminal is opened on and how: the step it follows, whether it skips permission
/// prompts, the model and effort asked for, the work its first line starts, and the kind of
/// session it runs in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TerminalOpening {
    pub target: TerminalTarget,
    pub step_id: Option<Id>,
    pub skip_permissions: bool,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub mode: TerminalMode,
    pub kind: SessionKind,
}
