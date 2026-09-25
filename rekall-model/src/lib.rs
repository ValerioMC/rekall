//! The entities and the state rules that live on them.
//!
//! One module per JPA entity, each a SeaORM entity over the table the migrations create, with
//! the rules the Java entity carried as methods on its `Model`: `TaskStep::markState`,
//! `Task::markReviewState`, `RunQueue::moveTo`, `RunQueueItem::moveTo`, and the bean-validation
//! constraints (`@NotBlank`, `@Size`, `@Pattern`) as a `validate` beside them.
//!
//! `@CreationTimestamp` / `@UpdateTimestamp` have no SeaORM equivalent that knows whether a row
//! changed, so the services stamp them when they write, and only when something did change.

pub mod constraints;
pub mod enums;
pub mod slug;

pub mod commit_reference;
pub mod company;
pub mod document;
pub mod document_task;
pub mod project;
pub mod run_queue;
pub mod run_queue_item;
pub mod tag;
pub mod task;
pub mod task_revision;
pub mod task_step;
pub mod time_entry;
pub mod wrapup;

pub use enums::*;
pub use slug::Slug;

pub mod prelude {
    pub use super::commit_reference::{Entity as CommitReference, Model as CommitReferenceModel};
    pub use super::company::{Entity as Company, Model as CompanyModel};
    pub use super::document::{Entity as Document, Model as DocumentModel};
    pub use super::document_task::{Entity as DocumentTask, Model as DocumentTaskModel};
    pub use super::project::{Entity as Project, Model as ProjectModel};
    pub use super::run_queue::{Entity as RunQueue, Model as RunQueueModel};
    pub use super::run_queue_item::{Entity as RunQueueItem, Model as RunQueueItemModel};
    pub use super::tag::{Entity as Tag, Model as TagModel};
    pub use super::task::{Entity as Task, Model as TaskModel};
    pub use super::task_revision::{Entity as TaskRevision, Model as TaskRevisionModel};
    pub use super::task_step::{Entity as TaskStep, Model as TaskStepModel};
    pub use super::time_entry::{Entity as TimeEntry, Model as TimeEntryModel};
    pub use super::wrapup::{Entity as Wrapup, Model as WrapupModel};
    pub use super::enums::*;
}
