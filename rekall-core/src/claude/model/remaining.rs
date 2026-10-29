/// Whether a session has anything to do on the task, and if not, why not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Remaining {
    /// An open or running step, or a stepless task not yet claimed.
    Open,
    /// Everything is claimed and at least part of it waits for review.
    Claimed,
    /// Everything has been accepted.
    Accepted,
}
