//! The rules a graph has to satisfy before it is stored. Every broken rule is reported, each
//! with the path to the element that broke it, so a session can fix them all in one pass.

mod graph_validator;
mod graph_violation;
mod graph_violations;

pub use graph_validator::{GraphValidator, MAX_EDGES, MAX_NODES};
pub use graph_violation::GraphViolation;
pub use graph_violations::GraphViolations;
