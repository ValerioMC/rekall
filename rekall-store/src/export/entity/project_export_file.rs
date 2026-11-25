//! The bytes of a finished export, one row per export, kept apart so a listing never reads them.

use rekall_kernel::Id;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "project_export_file")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub export_id: Id,
    pub bytes: Vec<u8>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "crate::project_export::Entity", from = "Column::ExportId", to = "crate::project_export::Column::Id", on_delete = "Cascade")]
    Export,
}

impl Related<crate::project_export::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Export.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
