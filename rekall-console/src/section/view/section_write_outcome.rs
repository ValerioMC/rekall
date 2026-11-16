/// What writing a section did: a title the project did not have, one it had and was rewritten, or one it had and was kept.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectionWriteOutcome {
    Created,
    Replaced,
    Kept,
}
