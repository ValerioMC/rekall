//! Which of the connections shared with a project's company a repository's `origin` belongs to.

use rekall_store::external_connection;

use crate::connection::support::repository_origin::RepositoryOrigin;

/// The connection at the origin's host and the repository path on it. A connection with a root path
/// covers only the repositories under it, so one that does not cover the repository is no candidate.
/// The longest root path wins, so `acme/platform` beats `acme`, which beats a connection to the whole
/// instance; among equals one with a credential is preferred, so an empty twin hides no usable one.
pub(in crate::connection) fn connection_at(git_hosts: &[external_connection::Model], origin: &RepositoryOrigin) -> Option<(external_connection::Model, String)> {
    let mut candidates: Vec<(&external_connection::Model, String, usize)> = git_hosts
        .iter()
        .filter_map(|connection| {
            let path = connection.base_url.as_deref().and_then(|url| origin.path_on(url))?;
            let covered = covered_length(connection.root_path.as_deref(), &path)?;
            Some((connection, path, covered))
        })
        .collect();
    candidates.sort_by_key(|(connection, _, covered)| (std::cmp::Reverse(*covered), !connection.has_credential()));
    candidates.into_iter().next().map(|(connection, path, _)| (connection.clone(), path))
}

/// How much of the repository path the root path covers, or none when the repository is outside it.
/// A root path matches whole segments, ignoring case: `acme` covers `acme/api` and not `acme-labs/api`.
fn covered_length(root_path: Option<&str>, repository_path: &str) -> Option<usize> {
    let Some(root_path) = root_path else {
        return Some(0);
    };
    let repository = repository_path.to_lowercase();
    let root = root_path.to_lowercase();
    (repository == root || repository.starts_with(&format!("{root}/"))).then_some(root.len())
}

#[cfg(test)]
#[path = "../../../tests/unit/connection/rule/repository_match_rules_tests.rs"]
mod tests;
