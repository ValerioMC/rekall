use rekall_kernel::Instant;

use super::{TokenCount, WorkUnit};

/// The tokens a session spent while it was on one unit of work, on one day.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitUsage {
    pub unit: WorkUnit,
    /// When the tokens were generated, or none for a line that carried no time.
    pub at: Option<Instant>,
    pub tokens: TokenCount,
}
