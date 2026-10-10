use super::{TokenCount, WorkUnit};

/// The tokens a session spent while it was on one unit of work.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitUsage {
    pub unit: WorkUnit,
    pub tokens: TokenCount,
}
