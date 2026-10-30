use rekall_core::claude::{SessionKind, TerminalMode, TerminalTarget};
use rekall_kernel::Id;

/// A terminal to open: on a task or a project, the step it follows, whether it skips permission
/// prompts, the model and effort asked for, the work its first line starts, and the kind of
/// session it runs in.
#[derive(Clone, Debug)]
pub struct OpenTerminalCommand {
    pub target: TerminalTarget,
    pub step_id: Option<Id>,
    pub skip_permissions: bool,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub mode: TerminalMode,
    pub kind: SessionKind,
}
