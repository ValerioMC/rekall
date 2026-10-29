use rekall_kernel::Id;

use super::{TerminalMode, TerminalTarget};

/// What a terminal is opened on and how: the step it follows, whether it skips permission
/// prompts, the model and effort asked for, and the work its first line starts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TerminalOpening {
    pub target: TerminalTarget,
    pub step_id: Option<Id>,
    pub skip_permissions: bool,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub mode: TerminalMode,
}
