//! The tokens agent sessions spent on each task and step. A reader of the transcripts hands
//! a session's spend over as `UnitUsage`; `TokenUsageService` resolves its anchors and keeps one
//! set of rows per session and day, and the console sums them per step, task and project, and
//! charts a task's spend day by day.

mod event;
mod model;
mod service;
mod view;

pub use model::token_count::TokenCount;
pub use view::token_day_view::TokenDayView;
pub use view::token_tally_view::TokenTallyView;
pub use service::token_usage_service::TokenUsageService;
pub use event::token_usage_stream_event::TokenUsageStreamEvent;
pub use model::unit_usage::UnitUsage;
pub use model::work_unit::WorkUnit;

const CODEX_SESSION_PREFIX: &str = "codex:";

pub fn codex_session_key(session_id: &str) -> String {
    format!("{CODEX_SESSION_PREFIX}{session_id}")
}

fn session_provider(session_id: &str) -> rekall_store::SessionProvider {
    use rekall_store::SessionProvider::{Claude, Codex};
    if session_id.starts_with(CODEX_SESSION_PREFIX) { Codex } else { Claude }
}
