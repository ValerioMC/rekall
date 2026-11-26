//! A picture a task's text shows as a task manager already holds it: which repository or Jira
//! issue it was uploaded to, and how that tracker refers to it.

use rekall_kernel::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::Violations;

pub const PICTURE_KEY_MAX_CHARACTERS: usize = 1_000;
pub const REMOTE_REFERENCE_MAX_CHARACTERS: usize = 2_000;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "tracker_picture")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub task_id: Id,
    #[sea_orm(primary_key, auto_increment = false)]
    pub destination: String,
    /// The path under the images folder, never the `/api/images/` address.
    #[sea_orm(primary_key, auto_increment = false)]
    pub picture_path: String,
    pub remote_reference: String,
    pub created_at: Instant,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::task::Entity",
        from = "Column::TaskId",
        to = "crate::task::Column::Id",
        on_delete = "Cascade"
    )]
    Task,
}

impl Related<crate::task::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn validate(&self) -> Result<(), RekallError> {
        Violations::new("TrackerPicture")
            .max_length("destination", Some(&self.destination), PICTURE_KEY_MAX_CHARACTERS)
            .max_length("picturePath", Some(&self.picture_path), PICTURE_KEY_MAX_CHARACTERS)
            .max_length("remoteReference", Some(&self.remote_reference), REMOTE_REFERENCE_MAX_CHARACTERS)
            .finish()
    }
}
