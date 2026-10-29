use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

string_enum!(
    /// How a note travels in a task's context: in full under every task it is on, or as its
    /// title, a line of it and an anchor a session loads only when the work needs it.
    DocumentContextMode { Full = "FULL", Reference = "REFERENCE" }
);
