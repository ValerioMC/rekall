/// What a reply came to: the entries it applied and the ones the task still has open, waiting or
/// in a pass.
#[derive(Clone, Debug)]
pub struct Replied {
    pub task_anchor: String,
    pub applied: Vec<i32>,
    pub still_open: Vec<i32>,
}
