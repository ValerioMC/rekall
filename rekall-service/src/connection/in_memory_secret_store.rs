use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

use rekall_common::{InternalFailure, RekallError, Result};

use super::{Secret, SecretStore};

/// The `SecretStore` tests use: a map in memory that forgets everything when dropped.
#[derive(Default)]
pub struct InMemorySecretStore {
    entries: Mutex<HashMap<String, Secret>>,
}

impl InMemorySecretStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn contains(&self, account: &str) -> bool {
        self.entries().is_ok_and(|entries| entries.contains_key(account))
    }

    fn entries(&self) -> Result<MutexGuard<'_, HashMap<String, Secret>>> {
        self.entries.lock().map_err(|_| RekallError::internal(InternalFailure::Background, "the in-memory secret store was poisoned"))
    }
}

impl SecretStore for InMemorySecretStore {
    fn put(&self, account: &str, secret: &Secret) -> Result<()> {
        self.entries()?.insert(account.to_string(), Secret::new(secret.expose()));
        Ok(())
    }

    fn read(&self, account: &str) -> Result<Secret> {
        self.entries()?
            .get(account)
            .map(|secret| Secret::new(secret.expose()))
            .ok_or_else(|| RekallError::not_found_msg(format!("No credential is stored under {account}.")))
    }

    fn remove(&self, account: &str) -> Result<()> {
        self.entries()?.remove(account);
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../tests/unit/connection/in_memory_secret_store_tests.rs"]
mod tests;
