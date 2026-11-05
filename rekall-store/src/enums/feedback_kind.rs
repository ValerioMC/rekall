use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

string_enum!(
    /// What a thread entry is about: the description rewritten, the work already done, or the plan.
    FeedbackKind {
        Revision = "REVISION",
        Implementation = "IMPLEMENTATION",
        Plan = "PLAN",
    }
);

impl FeedbackKind {
    pub fn is_revision(&self) -> bool {
        *self == Self::Revision
    }
}
