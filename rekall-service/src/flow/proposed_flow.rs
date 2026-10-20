/// What a proposed flow left on the task's shelf: its drafts and the routes between them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProposedFlow {
    pub steps: usize,
    pub routes: usize,
}
