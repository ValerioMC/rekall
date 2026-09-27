//! CODE → CONCEPT: from a line of code back to the elements of a graph it implements.
//! CONCEPT → CODE needs no index, a node carries its own sources.

mod trace_hit;
mod trace_index;

pub use trace_hit::TraceHit;
pub use trace_index::TraceIndex;
