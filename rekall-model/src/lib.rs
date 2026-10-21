//! The entities and the state rules that live on them.
//!
//! One module per table, each a SeaORM entity, grouped in a folder per domain and re-exported here, with the state rules of that row as methods on
//! its `Model` (`task_step::Model::mark_state`, `task::Model::mark_review_state`, the run queue's
//! moves) and its limits as a `validate` beside them.
//!
//! `updated_at` moves only when a value does: `Stamped` marks the rows that follow that rule, and
//! `rekall_repository::repository::update_if_changed` is the write that applies it.

pub mod constraints;
pub mod enums;
pub mod slug;
pub mod stamped;

pub mod catalog;
pub mod commit;
pub mod component;
pub mod connection;
pub mod diagram;
pub mod flow;
pub mod note;
pub mod queue;
pub mod revision;
pub mod section;
pub mod skill;
pub mod step;
pub mod tag;
pub mod task;
pub mod timeentry;
pub mod token;
pub mod wrapup;

pub use catalog::{company, company_logo_image, project, project_icon_image};
pub use commit::commit_reference;
pub use component::project_component;
pub use connection::{connection_access, external_connection, project_connection};
pub use flow::{step_link, step_visit};
pub use note::{document, document_task, note_scope};
pub use queue::{run_queue, run_queue_item};
pub use revision::task_revision;
pub use section::project_section;
pub use skill::{skill_name, skill_placement, tag_skill, task_skill};
pub use step::{step_pass, task_step};
pub use timeentry::time_entry;
pub use token::{token_usage, transcript_scan};
pub use enums::*;
pub use slug::Slug;
pub use stamped::Stamped;

pub mod prelude {
    pub use super::commit_reference::{Entity as CommitReference, Model as CommitReferenceModel};
    pub use super::company::{Entity as Company, Model as CompanyModel};
    pub use super::connection_access::{Entity as ConnectionAccess, Model as ConnectionAccessModel};
    pub use super::company_logo_image::{Entity as CompanyLogoImage, Model as CompanyLogoImageModel};
    pub use super::diagram::{Entity as Diagram, Model as DiagramModel};
    pub use super::document::{Entity as Document, Model as DocumentModel};
    pub use super::document_task::{Entity as DocumentTask, Model as DocumentTaskModel};
    pub use super::external_connection::{Entity as ExternalConnection, Model as ExternalConnectionModel};
    pub use super::project::{Entity as Project, Model as ProjectModel};
    pub use super::project_component::{Entity as ProjectComponent, Model as ProjectComponentModel};
    pub use super::project_connection::{Entity as ProjectConnection, Model as ProjectConnectionModel};
    pub use super::project_icon_image::{Entity as ProjectIconImage, Model as ProjectIconImageModel};
    pub use super::project_section::{Entity as ProjectSection, Model as ProjectSectionModel};
    pub use super::run_queue::{Entity as RunQueue, Model as RunQueueModel};
    pub use super::run_queue_item::{Entity as RunQueueItem, Model as RunQueueItemModel};
    pub use super::skill_placement::{Entity as SkillPlacement, Model as SkillPlacementModel};
    pub use super::step_link::{Entity as StepLink, Model as StepLinkModel};
    pub use super::step_visit::{Entity as StepVisit, Model as StepVisitModel};
    pub use super::tag::{Entity as Tag, Model as TagModel};
    pub use super::tag_skill::{Entity as TagSkill, Model as TagSkillModel};
    pub use super::task::{Entity as Task, Model as TaskModel};
    pub use super::task_revision::{Entity as TaskRevision, Model as TaskRevisionModel};
    pub use super::task_skill::{Entity as TaskSkill, Model as TaskSkillModel};
    pub use super::task_step::{Entity as TaskStep, Model as TaskStepModel};
    pub use super::time_entry::{Entity as TimeEntry, Model as TimeEntryModel};
    pub use super::token_usage::{Entity as TokenUsage, Model as TokenUsageModel};
    pub use super::transcript_scan::{Entity as TranscriptScan, Model as TranscriptScanModel};
    pub use super::wrapup::{Entity as Wrapup, Model as WrapupModel};
    pub use super::enums::*;
}
