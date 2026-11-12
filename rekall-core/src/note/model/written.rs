/// What a write came to: the note, where it landed and how many notes that task now carries.
#[derive(Clone, Debug)]
pub struct Written {
    pub title: String,
    pub note_anchor: String,
    pub task_anchor: String,
    pub notes_on_task: usize,
    pub replaced: bool,
}
