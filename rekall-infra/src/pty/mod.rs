//! Owns the `claude` pseudo-terminals behind the in-app terminal pane: one PTY per task, each
//! running the interactive `claude` TUI. A reader thread pumps PTY output to every attached
//! `TerminalListener` and into a bounded scrollback; `write` and `resize` carry input the other way. The
//! terminal moves the task's checklist marker (a step to `RUNNING` and back) or its review line as
//! it opens and closes, and announces every end, however it came, as a `TerminalEnded` event.
//! Nothing is persisted; strays are bounded by a cap, an idle sweep, and a shutdown hook.

mod claude_command;
mod pty_process;
mod pty_terminal_manager;
mod pty_terminals;
mod scrollback;
mod terminal;
mod terminal_ended;

pub use pty_terminal_manager::PtyTerminalManager;
pub use pty_terminals::PtyTerminals;
pub use terminal_ended::TerminalEnded;
