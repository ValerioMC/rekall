use rekall_common::Id;

/// The connection a step reaches, as the step's line in a context names it. `bound` says whether
/// the project still uses it, which is what puts it in the `<connections>` block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StepConnectionView {
    pub step_id: Id,
    pub label: String,
    pub bound: bool,
}
