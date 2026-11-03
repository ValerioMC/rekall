//! A file that was already modified when a snapshot was taken, with a hash of its content then.
//! A file that was already deleted has no hash.

use rekall_kernel::Id;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "step_snapshot_file")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub snapshot_id: Id,
    #[sea_orm(primary_key, auto_increment = false)]
    pub path: String,
    pub content_hash: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::step_snapshot::Entity",
        from = "Column::SnapshotId",
        to = "crate::step_snapshot::Column::Id",
        on_delete = "Cascade"
    )]
    StepSnapshot,
}

impl Related<crate::step_snapshot::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::StepSnapshot.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
