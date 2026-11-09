use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

string_enum!(
    /// What a description revision is applied to: the work already done, or the plan of steps.
    FeedbackKind {
        Implementation = "IMPLEMENTATION",
        Plan = "PLAN",
    }
);
