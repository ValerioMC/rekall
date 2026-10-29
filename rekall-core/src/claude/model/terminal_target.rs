use rekall_kernel::Id;

/// What a terminal is opened on. A task is where work happens; a project alone is where a
/// diagram is generated without a task to hang it on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalTarget {
    Task(Id),
    Project(Id),
}
