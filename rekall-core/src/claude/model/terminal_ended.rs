use rekall_kernel::Id;

/// A session has gone, whether it exited, was closed, idled out or went down with the app.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerminalEnded {
    pub terminal_id: Id,
    pub task_id: Option<Id>,
}
