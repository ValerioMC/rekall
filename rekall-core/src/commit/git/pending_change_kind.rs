#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PendingChangeKind {
    Added,
    Modified,
    Deleted,
    Renamed,
}
