/// Whether a host can merge a pull request now. `Unknown` while the host is still working it out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mergeability {
    Mergeable,
    Conflicting,
    Unknown,
}
