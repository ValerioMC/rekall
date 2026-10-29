//! The in-app terminals as the console drives them: opened on a task or project, listed, closed,
//! and the bytes of one piped to its pane. The terminals are reached through `TerminalHost` and
//! `TerminalPipe`.

mod open_terminal_command;
mod terminal_pipe_service;
mod terminal_service;

pub use open_terminal_command::OpenTerminalCommand;
pub use terminal_pipe_service::TerminalPipeService;
pub use terminal_service::TerminalService;
