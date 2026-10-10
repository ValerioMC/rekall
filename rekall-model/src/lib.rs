//! The entities and the state rules that live on them.
//!
//! One module per table, each a SeaORM entity, with the state rules of that row as methods on
//! its `Model` (`task_step::Model::mark_state`, `task::Model::mark_review_state`, the run queue's
//! moves) and its limits as a `validate` beside them.
//!
//! `updated_at` moves only when a value does: `Stamped` marks the rows that follow that rule, and
//! `rekall_repository::repository::update_if_changed` is the write that applies it.

pub mod constraints;
pub mod enums;
pub mod slug;
pub mod stamped;

pub mod commit_reference;
pub mod company;
pub mod company_logo_image;
pub mod diagram;
pub mod document;
pub mod document_task;
pub mod note_scope;
pub mod project;
pub mod project_component;
pub mod project_icon_image;
pub mod project_section;
pub mod run_queue;
pub mod run_queue_item;
pub mod step_pass;
pub mod tag;
pub mod task;
pub mod task_revision;
pub mod task_step;
pub mod time_entry;
pub mod token_usage;
pub mod transcript_scan;
pub mod wrapup;

pub use enums::*;
pub use slug::Slug;
pub use stamped::Stamped;

pub mod prelude {
    pub use super::commit_reference::{Entity as CommitReference, Model as CommitReferenceModel};
    pub use super::company::{Entity as Company, Model as CompanyModel};
    pub use super::company_logo_image::{Entity as CompanyLogoImage, Model as CompanyLogoImageModel};
    pub use super::diagram::{Entity as Diagram, Model as DiagramModel};
    pub use super::document::{Entity as Document, Model as DocumentModel};
    pub use super::document_task::{Entity as DocumentTask, Model as DocumentTaskModel};
    pub use super::project::{Entity as Project, Model as ProjectModel};
    pub use super::project_component::{Entity as ProjectComponent, Model as ProjectComponentModel};
    pub use super::project_icon_image::{Entity as ProjectIconImage, Model as ProjectIconImageModel};
    pub use super::project_section::{Entity as ProjectSection, Model as ProjectSectionModel};
    pub use super::run_queue::{Entity as RunQueue, Model as RunQueueModel};
    pub use super::run_queue_item::{Entity as RunQueueItem, Model as RunQueueItemModel};
    pub use super::tag::{Entity as Tag, Model as TagModel};
    pub use super::task::{Entity as Task, Model as TaskModel};
    pub use super::task_revision::{Entity as TaskRevision, Model as TaskRevisionModel};
    pub use super::task_step::{Entity as TaskStep, Model as TaskStepModel};
    pub use super::time_entry::{Entity as TimeEntry, Model as TimeEntryModel};
    pub use super::token_usage::{Entity as TokenUsage, Model as TokenUsageModel};
    pub use super::transcript_scan::{Entity as TranscriptScan, Model as TranscriptScanModel};
    pub use super::wrapup::{Entity as Wrapup, Model as WrapupModel};
    pub use super::enums::*;
}
