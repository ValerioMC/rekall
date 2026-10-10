//! The JSON form of a diagram document: how a session sends it and how it is stored.

mod document_codec;
mod document_format_error;

pub use document_codec::DocumentCodec;
pub use document_format_error::DocumentFormatError;
