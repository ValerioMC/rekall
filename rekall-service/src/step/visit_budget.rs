use rekall_common::{RekallError, Result};

/// The largest budget a step takes; a loop allowed more passes than this is not being bounded.
const MOST: i32 = 99;

/// How many times a flow may enter one step before a move into it is refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VisitBudget(i32);

impl VisitBudget {
    pub fn new(visits: i32) -> Result<Self> {
        if !(1..=MOST).contains(&visits) {
            return Err(RekallError::illegal(format!(
                "A step's visit budget runs from 1 to {MOST}, and {visits} was asked for. Clear it to set no limit."
            )));
        }
        Ok(Self(visits))
    }

    pub fn visits(self) -> i32 {
        self.0
    }
}

#[cfg(test)]
#[path = "../../tests/unit/step/visit_budget_tests.rs"]
mod tests;
