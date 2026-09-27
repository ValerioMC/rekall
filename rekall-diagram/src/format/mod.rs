//! The wire form of a [`crate::SemanticGraph`]: JSON with a format marker and a version, so a
//! stored graph says which reader it needs.

mod graph_codec;
mod graph_format_error;

pub use graph_codec::{GraphCodec, CURRENT_VERSION, FORMAT};
pub use graph_format_error::GraphFormatError;
