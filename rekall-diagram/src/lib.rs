//! The Semantic Graph: what a system does, told as concepts, states, decisions, events, data,
//! external systems, actions and code, and the relations between them.
//!
//! It knows nothing of the database or the console. A session writes it as JSON (`format`),
//! the application checks it (`validation`) and stores it as text, and the console draws it.
//! `trace` answers CODE → CONCEPT: which elements a line of code belongs to.

pub mod format;
pub mod graph;
pub mod trace;
pub mod validation;

pub use format::{GraphCodec, GraphFormatError, FORMAT, CURRENT_VERSION};
pub use graph::{GraphEdge, GraphNode, Metadata, NodeKind, Provenance, RelationKind, SemanticGraph, SourceLocation};
pub use trace::{TraceIndex, TraceHit};
pub use validation::{GraphValidator, GraphViolation, GraphViolations};
