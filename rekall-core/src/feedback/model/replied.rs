/// What a reply came to: the entries it applied and the ones the task still has pending.
#[derive(Clone, Debug)]
pub struct Replied {
    pub task_anchor: String,
    pub applied: Vec<i32>,
    pub still_pending: Vec<i32>,
}
