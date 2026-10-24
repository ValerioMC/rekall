use rekall_common::Id;

/// The connection a step reaches, as the step's line in a context names it. `bound_as` holds the
/// labels of the project's bindings of it, which are its lines in `<connections>`; none means the
/// project no longer uses it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StepConnectionView {
    pub step_id: Id,
    pub label: String,
    pub bound_as: Vec<String>,
}
