//! What every tool shares: the `project:` / `task:` anchor parsing, the task an anchor resolves to,
//! the argument reader and the refusals a tool answers with.

mod anchor;
mod anchored_project;
mod anchored_task;
mod arguments;
mod tool_failure;

pub use anchor::Anchor;
pub use anchored_project::AnchoredProject;
pub use anchored_task::AnchoredTask;
pub use arguments::Arguments;
pub(crate) use tool_failure::{told, QUALIFY_WITH_PROJECT};
