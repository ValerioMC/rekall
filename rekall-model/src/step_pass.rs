//! One earlier pass at a step: the detail a session worked from, kept when the console sent the
//! claim back, so the next pass reads what was done before and the feedback apart.

use rekall_common::Instant;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepPass {
    /// The step's detail as that pass read it: the brief for the first, the feedback after that.
    pub detail_markdown: Option<String>,
    pub sent_back_at: Instant,
}
