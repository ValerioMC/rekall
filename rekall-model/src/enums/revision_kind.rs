use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

string_enum!(
    /// Which text of a task a revision keeps an earlier version of.
    RevisionKind { Wrapup = "WRAPUP", Description = "DESCRIPTION" }
);
