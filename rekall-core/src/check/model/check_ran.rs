use std::time::Duration;

/// A check that passed, with how long it took.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckRan {
    pub label: String,
    pub duration: Duration,
}
