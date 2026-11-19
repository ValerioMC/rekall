use rekall_kernel::Id;
use rekall_store::DocumentContextMode;

/// `contextMode` left out keeps what the note had, or `FULL` for a new one. A note belongs to
/// the task it was written on: `task_id` on an update has to be that task.
#[derive(Clone, Debug)]
pub struct DocumentCommand {
    pub task_id: Id,
    pub title: Option<String>,
    pub kind: Option<String>,
    pub body_markdown: Option<String>,
    pub context_mode: Option<DocumentContextMode>,
}
