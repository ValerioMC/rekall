//! A company a connection is shared with. A project can bind a connection only when its company
//! is one of these; the connection itself belongs to no company.

use rekall_kernel::Id;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "connection_company")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub connection_id: Id,
    #[sea_orm(primary_key, auto_increment = false)]
    pub company_id: Id,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::external_connection::Entity",
        from = "Column::ConnectionId",
        to = "crate::external_connection::Column::Id",
        on_delete = "Cascade"
    )]
    Connection,
    #[sea_orm(belongs_to = "crate::company::Entity", from = "Column::CompanyId", to = "crate::company::Column::Id", on_delete = "Cascade")]
    Company,
}

impl Related<crate::external_connection::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Connection.def()
    }
}

impl Related<crate::company::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Company.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
