//! The limits a row has to satisfy before it is written. SQLite enforces no column length, so
//! every cap is checked here, in UTF-16 code units like every other text limit in Rekall.

mod violations;

pub use violations::Violations;
