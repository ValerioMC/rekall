use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

string_enum!(
    /// Where a failed export stopped: writing the document, or keeping the file.
    ExportFailureKind {
        Render = "RENDER",
        Storage = "STORAGE",
    }
);
