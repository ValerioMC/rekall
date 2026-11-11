/// What a description write came to: the task, and whether the text differed from the one it had.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DescriptionReplaced {
    pub task_anchor: String,
    pub changed: bool,
}
