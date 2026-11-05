use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

string_enum!(
    /// A thread entry waits for a session (`PENDING`), is taken into a pass by Run revision (`RUNNING`),
    /// and stays there until a session replies to it (`APPLIED`).
    FeedbackState {
        Pending = "PENDING",
        Running = "RUNNING",
        Applied = "APPLIED",
    }
);
