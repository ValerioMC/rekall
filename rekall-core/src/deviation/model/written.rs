/// What a write came to: where the deviation landed and how many the task now carries.
#[derive(Clone, Debug)]
pub struct Written {
    pub task_anchor: String,
    pub step_number: i32,
    pub step_title: String,
    pub replaced: bool,
    pub deviations_on_task: usize,
}
