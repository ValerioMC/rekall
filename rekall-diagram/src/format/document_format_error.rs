use std::fmt;

/// Why a text could not be read as a diagram document at all, before any rule about its
/// content: not JSON, or JSON of the wrong shape. The detail says where.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentFormatError(pub String);

impl fmt::Display for DocumentFormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Not a diagram document: {}", self.0)
    }
}

impl std::error::Error for DocumentFormatError {}
