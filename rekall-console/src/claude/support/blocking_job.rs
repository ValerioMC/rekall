use rekall_kernel::{InternalFailure, RekallError, Result};
use tracing::error;

/// Runs blocking work off the async workers. A panic or a cancellation of the job is an internal
/// error, logged with its cause, never a panic of the caller.
pub(in crate::claude) async fn blocking<T: Send + 'static>(work: impl FnOnce() -> Result<T> + Send + 'static) -> Result<T> {
    match tokio::task::spawn_blocking(work).await {
        Ok(outcome) => outcome,
        Err(failure) => {
            error!(%failure, "A blocking job panicked or was cancelled");
            Err(RekallError::internal(InternalFailure::Background, failure))
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/claude/support/blocking_job_tests.rs"]
mod tests;
