use std::sync::Arc;

use rekall_kernel::{RekallError, Result};
use tracing::warn;

use crate::connection::{Secret, SecretStore};

/// Keeps credentials in `current` and reaches into `previous` only for an account `current` does
/// not hold yet, copying what it finds across. `previous` is asked once per account, so its prompts
/// end after the first read. A removal reaches both, or the old copy would come back on the next read.
pub struct MigratingSecretStore {
    current: Arc<dyn SecretStore>,
    previous: Arc<dyn SecretStore>,
}

impl MigratingSecretStore {
    pub fn new(current: Arc<dyn SecretStore>, previous: Arc<dyn SecretStore>) -> Self {
        Self { current, previous }
    }

    fn migrate(&self, account: &str) -> Result<Secret> {
        let secret = self.previous.read(account)?;
        if let Err(failure) = self.current.put(account, &secret) {
            warn!(%failure, account, "A credential read from the previous store could not be copied to the current one");
        }
        Ok(secret)
    }
}

impl SecretStore for MigratingSecretStore {
    fn put(&self, account: &str, secret: &Secret) -> Result<()> {
        self.current.put(account, secret)
    }

    fn read(&self, account: &str) -> Result<Secret> {
        match self.current.read(account) {
            Err(RekallError::NotFound(_)) => self.migrate(account),
            found => found,
        }
    }

    fn remove(&self, account: &str) -> Result<()> {
        self.current.remove(account)?;
        self.previous.remove(account)
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/connection/port/migrating_secret_store_tests.rs"]
mod tests;
