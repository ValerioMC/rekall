//! Owns the `claude` pseudo-terminals behind the in-app terminal pane: one PTY per task, each
//! running the interactive `claude` TUI. A reader thread pumps PTY output to every attached
//! `Listener` and into a bounded scrollback; `write` and `resize` carry input the other way. The
//! terminal moves the task's checklist marker (a step to `RUNNING` and back) or its review line as
//! it opens and closes, and announces every end, however it came, as a `TerminalEnded` event.
//! Nothing is persisted; strays are bounded by a cap, an idle sweep, and a shutdown hook.

mod listener;
mod pty_terminal_manager;
mod scrollback;
mod terminal;
mod terminal_ended;
mod terminal_view;

pub use listener::Listener;
pub use pty_terminal_manager::{PtyTerminalManager, EFFORT_LEVELS, MODEL_ALIASES};
pub use terminal_ended::TerminalEnded;
pub use terminal_view::TerminalView;
