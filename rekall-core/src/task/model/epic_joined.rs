/// What filing a task under an epic came to: both anchors, and the epic it left, if it was in one.
#[derive(Clone, Debug)]
pub struct EpicJoined {
    pub task_anchor: String,
    pub epic_anchor: String,
    pub epic_title: String,
    pub left: Option<String>,
}
