use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

string_enum!(
    /// The file an export is written as.
    ExportFormat {
        Pdf = "PDF",
        Docx = "DOCX",
    }
);
