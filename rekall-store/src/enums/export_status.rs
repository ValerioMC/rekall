use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

string_enum!(
    /// Whether the export produced a stored file.
    ExportStatus {
        Done = "DONE",
        Failed = "FAILED",
    }
);
