use rekall_kernel::{Id, InternalFailure, RekallError, Result};

/// The record `id` names, cloned from rows the caller has just read. Missing is an internal error
/// stating the `invariant` the caller had established.
pub fn require_by_id<T: Clone>(records: &[T], id: Id, id_of: impl Fn(&T) -> Id, invariant: &str) -> Result<T> {
    records
        .iter()
        .find(|record| id_of(record) == id)
        .cloned()
        .ok_or_else(|| RekallError::internal(InternalFailure::Integrity, invariant))
}

#[cfg(test)]
#[path = "../../tests/unit/shared/record_lookup_tests.rs"]
mod tests;
