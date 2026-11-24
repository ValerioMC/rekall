use crate::workingset::{OpenTaskView, PlaceView};

/// The open tasks in the order they were opened, each with its project, and the place last seen in
/// every project that has one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkingSetView {
    pub tasks: Vec<OpenTaskView>,
    pub places: Vec<PlaceView>,
}
