//! What every layer shares: the error vocabulary (`RekallError`), the two value types every row
//! carries (`Id`, `Instant`), and the text rules every length cap is measured with (`jstr`).

pub mod error;
pub mod id;
pub mod instant;
pub mod jstr;

pub use error::{InternalFailure, RekallError, Result};
pub use id::Id;
pub use instant::Instant;
