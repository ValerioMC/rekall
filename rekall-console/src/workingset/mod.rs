//! The tasks a person keeps open and the last place seen in each project. Only a person opening or
//! closing a task changes the set; no session, queue or review does.

mod rule;
mod service;
mod view;

pub use service::working_set_service::WorkingSetService;
pub use view::open_task_view::OpenTaskView;
pub use view::place_view::PlaceView;
pub use view::working_set_view::WorkingSetView;
