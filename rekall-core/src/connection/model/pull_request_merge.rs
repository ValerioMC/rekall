/// What a host answered to a merge. `Conflicting` is for the session to resolve; `Refused` carries
/// the host's own reason (a protected branch, a merge method it does not allow).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PullRequestMerge {
    Merged,
    Conflicting,
    Refused(String),
}
