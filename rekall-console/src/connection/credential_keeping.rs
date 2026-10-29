//! The credential store's calls, moved off the async runtime: a keychain may block on a prompt.

use std::sync::Arc;

use rekall_common::{InternalFailure, RekallError, Result};
use rekall_service::connection::{Secret, SecretStore};
use tracing::warn;

pub(super) async fn keep(secrets: Arc<dyn SecretStore>, account: String, secret: Secret) -> Result<()> {
    blocking(move || secrets.put(&account, &secret)).await
}

pub async fn recall(secrets: Arc<dyn SecretStore>, account: String) -> Result<Secret> {
    blocking(move || secrets.read(&account)).await
}

pub(super) async fn forget(secrets: Arc<dyn SecretStore>, account: String) -> Result<()> {
    blocking(move || secrets.remove(&account)).await
}

/// After the row is gone nothing names the account again, so a failure is logged, not returned.
pub(super) async fn forget_leftover(secrets: Arc<dyn SecretStore>, account: String) {
    if let Err(failure) = forget(secrets, account.clone()).await {
        warn!(%failure, account, "A deleted connection's credential could not be removed from the OS credential store");
    }
}

pub(super) async fn blocking<T: Send + 'static>(work: impl FnOnce() -> Result<T> + Send + 'static) -> Result<T> {
    tokio::task::spawn_blocking(work).await.map_err(|failure| RekallError::internal(InternalFailure::Background, failure))?
}
