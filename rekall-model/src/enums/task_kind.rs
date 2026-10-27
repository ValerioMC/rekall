use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

string_enum!(
    /// A plain task, or an epic: a task that holds no steps and gathers other tasks under it.
    TaskKind {
        Task = "TASK",
        Epic = "EPIC"
    }
);
