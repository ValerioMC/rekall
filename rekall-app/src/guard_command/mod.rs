//! `rekall-server guard before-tool|before-stop`: the program Claude Code runs as a hook. It reads
//! the event from standard input, asks the running Rekall for the context, lets `GuardService`
//! decide and answers with the exit code Claude Code understands: 0 lets the action go ahead, 2
//! blocks it and sends the reason on standard error.

mod context_source;
mod guard_cli;
mod guard_mode;
mod guard_outcome;
mod guard_runner;
mod hook_input;

pub use context_source::{ContextUnavailable, GuardContextSource, LocalRekall};
pub use guard_cli::execute;
pub use guard_mode::GuardMode;
pub use guard_outcome::GuardOutcome;
pub use guard_runner::run;
