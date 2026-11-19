use rekall_core::claude::{SessionKind, SessionProvider};
use rekall_kernel::Id;

/// What to do, in the person's words, and how the session that does it is opened.
#[derive(Clone, Debug)]
pub struct QuickTaskCommand {
    pub project_id: Id,
    pub text: String,
    pub provider: SessionProvider,
    pub skip_permissions: bool,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub kind: SessionKind,
}
