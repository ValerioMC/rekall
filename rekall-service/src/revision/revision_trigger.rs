/// What is about to replace a task's text, which decides whether the text it replaces is kept.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RevisionTrigger {
    /// The console's editor saving as someone types: kept once per editing window.
    HandEdit,
    /// A session writing over MCP: always kept.
    ClaudeWrite,
    /// The text is being removed: always kept.
    Deletion,
    /// An earlier revision is being written back: always kept, so the restore can itself be undone.
    Restore,
}
