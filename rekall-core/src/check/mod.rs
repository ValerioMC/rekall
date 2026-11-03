//! The checks a person sets to pass before a claim is accepted. Their rows are written by the
//! console alone; here a step's checks are run, one at a time, through the `CheckRunner` port, and
//! each run is kept.

mod event;
mod model;
mod port;
pub mod rule;
mod service;

pub use event::check_stream_event::CheckStreamEvent;
pub use model::check_outcome::CheckOutcome;
pub use model::check_ran::CheckRan;
pub use model::check_report::CheckReport;
pub use model::check_run::CheckRun;
pub use port::check_runner::CheckRunner;
pub use port::scripted_check_runner::ScriptedCheckRunner;
pub use service::check_execution_service::CheckExecutionService;
