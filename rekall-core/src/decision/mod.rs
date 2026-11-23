//! The one way a session records what is settled about a task before it runs: a question with its
//! options, an answer, later corrections and details, or a hold while the person finds out. The
//! task's context carries the current state of every decision.

mod command;
mod event;
pub mod rule;
mod service;
mod view;

pub use rekall_store::{DecisionSource, DecisionState, Speaker};

pub use command::answer_decision::AnswerDecision;
pub use command::ask_decision::AskDecision;
pub use command::correct_decision::CorrectDecision;
pub use command::offered_option::OfferedOption;
pub use command::spoken_line::SpokenLine;
pub use event::decision_stream_event::DecisionStreamEvent;
pub use service::decision_service::DecisionService;
pub use view::decision_detail_view::DecisionDetailView;
pub use view::decision_line_view::DecisionLineView;
pub use view::decision_option_view::DecisionOptionView;
pub use view::decision_view::DecisionView;
