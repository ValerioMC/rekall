//! A command a person set to run before a claim is accepted: for every step of the project, or for
//! one step. Only the console writes it, since a session able to write its own check could write
//! one that always passes.

use rekall_kernel::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::Violations;

pub const LABEL_MAX_CHARACTERS: usize = 100;
pub const COMMAND_MAX_CHARACTERS: usize = 2_000;
pub const FOLDER_MAX_CHARACTERS: usize = 500;
pub const TIMEOUT_MIN_SECONDS: i32 = 1;
pub const TIMEOUT_MAX_SECONDS: i32 = 3_600;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "done_check")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub project_id: Id,
    /// Absent when the check applies to every step of the project.
    pub step_id: Option<Id>,
    pub label: String,
    pub command: String,
    /// Relative to the project's folder; absent means the folder itself.
    pub folder: Option<String>,
    pub timeout_seconds: i32,
    /// Zero-based place among the checks of the same scope.
    pub position: i32,
    pub created_at: Instant,
    pub updated_at: Instant,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "crate::project::Entity", from = "Column::ProjectId", to = "crate::project::Column::Id", on_delete = "Cascade")]
    Project,
    #[sea_orm(belongs_to = "crate::task_step::Entity", from = "Column::StepId", to = "crate::task_step::Column::Id", on_delete = "Cascade")]
    TaskStep,
}

impl Related<crate::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl Related<crate::task_step::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TaskStep.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn validate(&self) -> Result<(), RekallError> {
        Violations::new("Check")
            .not_blank("label", Some(&self.label))
            .max_length("label", Some(&self.label), LABEL_MAX_CHARACTERS)
            .not_blank("command", Some(&self.command))
            .max_length("command", Some(&self.command), COMMAND_MAX_CHARACTERS)
            .max_length("folder", self.folder.as_deref(), FOLDER_MAX_CHARACTERS)
            .in_range("timeoutSeconds", self.timeout_seconds, TIMEOUT_MIN_SECONDS, TIMEOUT_MAX_SECONDS)
            .finish()
    }
}
