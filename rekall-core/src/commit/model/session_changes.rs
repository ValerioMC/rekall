use crate::commit::PendingChange;

/// The pending changes sorted by whose they are when a step or task is claimed: the ones the
/// session made, and the ones that were already in the folder, unchanged, when the work started.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionChanges {
    pub own: Vec<PendingChange>,
    pub left_out: Vec<PendingChange>,
}

impl SessionChanges {
    /// `hashes` runs parallel to `pending`. A change is left out when the snapshot holds its path
    /// with the same content hash, where a missing hash on both sides counts as the same.
    pub fn split(pending: Vec<PendingChange>, hashes: Vec<Option<String>>, snapshot: &[(String, Option<String>)]) -> Self {
        let mut own = Vec::new();
        let mut left_out = Vec::new();
        for (change, hash) in pending.into_iter().zip(hashes) {
            let unchanged = snapshot.iter().any(|(path, kept)| *path == change.path && *kept == hash);
            if unchanged {
                left_out.push(change);
            } else {
                own.push(change);
            }
        }
        Self { own, left_out }
    }

    /// With no snapshot to compare against, everything pending is taken as the session's.
    pub fn everything(pending: Vec<PendingChange>) -> Self {
        Self { own: pending, left_out: Vec::new() }
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/commit/model/session_changes_tests.rs"]
mod tests;
