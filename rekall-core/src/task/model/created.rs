use rekall_store::TaskKind;

/// What a creation came to: the task, where it landed and the open steps and routes it carries.
#[derive(Clone, Debug)]
pub struct Created {
    pub title: String,
    pub kind: TaskKind,
    pub task_anchor: String,
    pub steps: usize,
    pub routes: usize,
}
