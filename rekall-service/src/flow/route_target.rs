use rekall_common::Id;

/// Where a route leads: another step of the task, or the end of the flow.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouteTarget {
    Step(Id),
    End,
}

impl RouteTarget {
    /// The step it names, `None` for the end.
    pub fn step_id(self) -> Option<Id> {
        match self {
            Self::Step(id) => Some(id),
            Self::End => None,
        }
    }
}
