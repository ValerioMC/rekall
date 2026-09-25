//! What every layer shares: the error vocabulary (`NotFound`, `Conflict` and the rest the Java
//! code spelled as exception classes), the two value types every row carries (`Id`, `Instant`),
//! and the handful of `java.lang.String` semantics the domain rules were written against.

pub mod error;
pub mod id;
pub mod instant;
pub mod jcoll;
pub mod jstr;

pub use error::{RekallError, Result};
pub use id::Id;
pub use instant::Instant;
