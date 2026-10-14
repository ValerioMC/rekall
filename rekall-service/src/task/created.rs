/// What a creation came to: the task, where it landed and how many drafts it carries.
#[derive(Clone, Debug)]
pub struct Created {
    pub title: String,
    pub task_anchor: String,
    pub drafts: usize,
}
