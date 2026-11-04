//! The one way a session records where its work departed from the plan: a deviation is written on
//! one task for one of its steps, and writing it again for the same step rewrites it. It is not a
//! note: it never appears among a task's notes, and no session can delete one.

mod event;
mod model;
mod rule;
mod service;
mod view;

pub use event::deviation_stream_event::DeviationStreamEvent;
pub use model::written::Written;
pub use rule::planned_step_rule::step_at;
pub use service::deviation_service::DeviationService;
pub use view::deviation_view::DeviationView;
