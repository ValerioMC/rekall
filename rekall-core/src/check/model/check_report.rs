use super::check_outcome::CheckOutcome;
use super::check_ran::CheckRan;

/// What running a step's checks found. A run stops at the first check that fails, so `Failed`
/// names that one and the checks after it were not run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckReport {
    NoChecks,
    Passed { runs: Vec<CheckRan> },
    Failed { label: String, outcome: CheckOutcome, output_tail: String },
}
