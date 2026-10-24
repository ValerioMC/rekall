use serde::Serialize;

/// What discovering a repository did with it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DiscoveryOutcome {
    /// A new binding reaches its GitLab project.
    Bound,
    /// The project already binds its GitLab project, by hand, on an earlier run or for another clone.
    AlreadyBound,
    /// It has no `origin` remote, or one that names no host and project.
    NoOrigin,
    /// No GitLab connection of the company is at its origin's host.
    NoConnection,
    /// The connection at its host has no credential stored, so nothing is bound.
    NoCredential,
}
