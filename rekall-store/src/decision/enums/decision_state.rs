use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

string_enum!(
    /// Where a decision stands: asked and unanswered, answered, or put aside while the person finds out.
    DecisionState {
        Open = "OPEN",
        Answered = "ANSWERED",
        OnHold = "ON_HOLD",
    }
);
