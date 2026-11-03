/// What Claude Code reports to a hook. `command_or_path` is the shell command of a shell tool, or
/// the path or pattern another tool works on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuardEvent {
    BeforeTool { tool_name: String, command_or_path: String },
    BeforeStop,
}
