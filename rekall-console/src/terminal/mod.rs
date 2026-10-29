//! The in-app terminals as the console drives them: opened on a task or project, listed, closed,
//! and the bytes of one piped to its pane. The terminals are reached through `TerminalHost` and
//! `TerminalPipe`.

mod command;
mod service;

pub use command::open_terminal_command::OpenTerminalCommand;
pub use service::terminal_pipe_service::TerminalPipeService;
pub use service::terminal_service::TerminalService;
