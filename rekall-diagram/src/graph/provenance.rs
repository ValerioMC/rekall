use serde::{Deserialize, Serialize};

/// How an element was arrived at, which is how far a reader should trust it without opening
/// the code: `Observed` is in the code as written, `Inferred` is the session's reading of it,
/// `Documented` comes from comments or docs, `Stated` from what a person said.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Provenance {
    Observed,
    Inferred,
    Documented,
    Stated,
}
