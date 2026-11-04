//! How closely the work followed the plan: steps accepted at the first pass, steps sent back,
//! deviations and flow visits. Read-only, over data other domains write.

mod service;
mod view;

pub use service::plan_adherence_service::PlanAdherenceService;
pub use view::adherence_counts::AdherenceCounts;
pub use view::plan_adherence_view::PlanAdherenceView;
pub use view::project_adherence_view::ProjectAdherenceView;
pub use view::step_visits_view::StepVisitsView;
pub use view::task_sent_back_view::TaskSentBackView;
