/// What deleting a connection took with it.
#[derive(Clone, Copy, Debug)]
pub struct ConnectionRemovedView {
    pub unbound_projects: usize,
}
