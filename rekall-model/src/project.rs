//! Something worked on over time, inside one company. `label` is unique per company and is the
//! anchor; `title` is free text.

use rekall_common::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::{Phase, Violations};
use crate::enums::ProjectStatus;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "project")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub label: String,
    pub title: String,
    pub status: ProjectStatus,
    pub description: Option<String>,
    pub blueprint_markdown: Option<String>,
    pub repo_folder: Option<String>,
    /// Whether a session's claim commits the repo folder for it. Meaningful only while
    /// `repo_folder` is a git repository; the catalog clears it otherwise.
    pub auto_commit: bool,
    pub company_id: Id,
    pub created_at: Instant,
    pub updated_at: Instant,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::company::Entity",
        from = "Column::CompanyId",
        to = "super::company::Column::Id",
        on_delete = "Cascade"
    )]
    Company,
    #[sea_orm(has_many = "super::task::Entity")]
    Task,
}

impl Related<super::company::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Company.def()
    }
}

impl Related<super::task::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn anchor(&self) -> String {
        format!("project:{}", self.label)
    }

    pub fn validate(&self, phase: Phase) -> Result<(), RekallError> {
        Violations::new("Project", phase)
            .not_blank("label", Some(&self.label))
            .size("label", Some(&self.label), 120)
            .slug("label", Some(&self.label))
            .not_blank("title", Some(&self.title))
            .size("title", Some(&self.title), 200)
            .size("repoFolder", self.repo_folder.as_deref(), 1_000)
            .column("description", self.description.as_deref(), 100_000)
            .column("blueprint_markdown", self.blueprint_markdown.as_deref(), 100_000)
            .finish()
    }
}
