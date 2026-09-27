use std::fmt;

/// Why a text could not be read as a Semantic Graph at all, before any rule about its content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GraphFormatError {
    /// Not JSON, or JSON of the wrong shape; the text says where.
    Malformed(String),
    UnknownFormat(String),
    UnsupportedVersion(u32),
}

impl fmt::Display for GraphFormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GraphFormatError::Malformed(detail) => write!(f, "Not a Semantic Graph: {detail}"),
            GraphFormatError::UnknownFormat(format) => {
                write!(f, "Unknown format \"{format}\"; a Semantic Graph declares \"{}\"", super::FORMAT)
            }
            GraphFormatError::UnsupportedVersion(version) => {
                write!(f, "Semantic Graph version {version} is not supported; this build reads version {}", super::CURRENT_VERSION)
            }
        }
    }
}

impl std::error::Error for GraphFormatError {}
