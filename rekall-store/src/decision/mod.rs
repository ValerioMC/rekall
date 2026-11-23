//! What a person and a session settle about a task before it runs: one record per question, its
//! options, the conversation lines behind it and the details added later.

pub mod entity;
pub mod enums;
pub mod repository;

mod record;

pub use record::DecisionRecord;
