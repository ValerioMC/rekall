/// What a write came to: the note, where it landed, how many notes that task now carries, and,
/// on a replace, how many other tasks read the rewritten body too.
#[derive(Clone, Debug)]
pub struct Written {
    pub title: String,
    pub note_anchor: String,
    pub task_anchor: String,
    pub notes_on_task: usize,
    pub replaced: bool,
    pub other_tasks: usize,
}
