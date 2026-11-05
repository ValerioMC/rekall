use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

string_enum!(
    /// A thread entry waits for a session (`PENDING`) until one replies to it (`APPLIED`).
    FeedbackState {
        Pending = "PENDING",
        Applied = "APPLIED",
    }
);
