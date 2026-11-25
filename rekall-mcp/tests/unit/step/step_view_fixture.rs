use rekall_core::step::{TaskStepState, TaskStepView};
use rekall_kernel::{Id, Instant};

pub fn step(position: i32, title: &str, state: TaskStepState, detail: Option<&str>) -> TaskStepView {
    let now = Instant::now();
    TaskStepView {
        id: Id::random(),
        task_id: Id::random(),
        title: title.into(),
        body_markdown: detail.map(str::to_string),
        state,
        done: state == TaskStepState::Done,
        running_at: None,
        claimed_at: None,
        done_at: None,
        position,
        flow_x: None,
        flow_y: None,
        max_visits: None,
        starts_session: false,
        connection_id: None,
        passes: Vec::new(),
        links: Vec::new(),
        visits: Vec::new(),
        created_at: now,
        updated_at: now,
    }
}
