use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use rekall_kernel::{InternalFailure, RekallError, Result};

use super::{Secret, SecretStore};

/// Reads each account from the store it wraps once, then answers from memory until the account is
/// written or removed. The macOS Keychain may prompt on every read, so this turns one prompt per
/// `kubectl` or `git` call into one per connection per launch. A change made outside Rekall is seen
/// after a restart.
pub struct RememberingSecretStore {
    store: Arc<dyn SecretStore>,
    remembered: Mutex<HashMap<String, Secret>>,
}

impl RememberingSecretStore {
    pub fn new(store: Arc<dyn SecretStore>) -> Self {
        Self { store, remembered: Mutex::new(HashMap::new()) }
    }

    fn remembered(&self) -> Result<MutexGuard<'_, HashMap<String, Secret>>> {
        self.remembered.lock().map_err(|_| RekallError::internal(InternalFailure::Background, "the remembered secrets were poisoned"))
    }
}

/// Every call holds the lock across the wrapped store, so helpers asking for one account at the
/// same moment raise one prompt, not one each.
impl SecretStore for RememberingSecretStore {
    fn put(&self, account: &str, secret: &Secret) -> Result<()> {
        let mut remembered = self.remembered()?;
        remembered.remove(account);
        self.store.put(account, secret)?;
        remembered.insert(account.to_string(), Secret::new(secret.expose()));
        Ok(())
    }

    fn read(&self, account: &str) -> Result<Secret> {
        let mut remembered = self.remembered()?;
        if let Some(secret) = remembered.get(account) {
            return Ok(Secret::new(secret.expose()));
        }
        let secret = self.store.read(account)?;
        remembered.insert(account.to_string(), Secret::new(secret.expose()));
        Ok(secret)
    }

    fn remove(&self, account: &str) -> Result<()> {
        let mut remembered = self.remembered()?;
        remembered.remove(account);
        self.store.remove(account)
    }
}

#[cfg(test)]
#[path = "../../tests/unit/connection/remembering_secret_store_tests.rs"]
mod tests;
