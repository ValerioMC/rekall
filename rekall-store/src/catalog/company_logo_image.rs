//! The square PNG a company shows as its logo, already normalised by the server (see
//! `CompanyLogoService`). One row per company; absent means the company wears its initial.

use rekall_kernel::{Id, Instant};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "company_logo_image")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub company_id: Id,
    pub png: Vec<u8>,
    /// Also the logo's version: it changes with every upload, so a URL carrying it is immutable.
    pub updated_at: Instant,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "crate::company::Entity", from = "Column::CompanyId", to = "crate::company::Column::Id", on_delete = "Cascade")]
    Company,
}

impl Related<crate::company::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Company.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
