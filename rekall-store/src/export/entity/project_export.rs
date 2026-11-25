//! One export of a project: what was asked, the file's name and size, and when it failed, where
//! and why. The file itself is `project_export_file`, so a listing never reads it.

use rekall_kernel::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::Violations;
use crate::enums::{ExportFailureKind, ExportFormat, ExportStatus};

pub const SUMMARY_MAX_CHARACTERS: usize = 300;
pub const FILE_NAME_MAX_CHARACTERS: usize = 255;
pub const FAILURE_MESSAGE_MAX_CHARACTERS: usize = 1_000;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "project_export")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub project_id: Id,
    pub format: ExportFormat,
    pub status: ExportStatus,
    /// What was exported, in a line: the item's title, or the blueprint, page and component counts.
    pub summary: String,
    pub file_name: String,
    pub size_bytes: i64,
    /// The mermaid diagrams the document holds an error paragraph for.
    pub diagrams_missing: i32,
    /// Set exactly when the export failed.
    pub failure_kind: Option<ExportFailureKind>,
    pub failure_message: Option<String>,
    pub created_at: Instant,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "crate::project::Entity", from = "Column::ProjectId", to = "crate::project::Column::Id", on_delete = "Cascade")]
    Project,
    #[sea_orm(has_one = "crate::project_export_file::Entity")]
    File,
}

impl Related<crate::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl Related<crate::project_export_file::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::File.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn validate(&self) -> Result<(), RekallError> {
        let failed = self.status == ExportStatus::Failed;
        Violations::new("ProjectExport")
            .not_blank("summary", Some(&self.summary))
            .max_length("summary", Some(&self.summary), SUMMARY_MAX_CHARACTERS)
            .not_blank("fileName", Some(&self.file_name))
            .max_length("fileName", Some(&self.file_name), FILE_NAME_MAX_CHARACTERS)
            .max_length("failureMessage", self.failure_message.as_deref(), FAILURE_MESSAGE_MAX_CHARACTERS)
            .require(self.size_bytes >= 0, "sizeBytes must not be negative")
            .require(self.diagrams_missing >= 0, "diagramsMissing must not be negative")
            .require(
                failed == (self.failure_kind.is_some() && self.failure_message.is_some()),
                "a failed export carries its failure kind and message, a done one carries neither",
            )
            .require(failed || (self.failure_kind.is_none() && self.failure_message.is_none()), "a done export carries no failure")
            .finish()
    }
}
