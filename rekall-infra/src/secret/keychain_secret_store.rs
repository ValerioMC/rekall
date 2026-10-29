use keyring::Entry;
use rekall_kernel::{InternalFailure, RekallError, Result};
use rekall_core::connection::{Secret, SecretStore};
use tracing::error;

pub const KEYCHAIN_SERVICE: &str = "Rekall-connections";

/// The OS credential store through `keyring`: the macOS Keychain, the Windows Credential Manager,
/// or the Linux kernel keyring, which forgets its entries when the session ends.
pub struct KeychainSecretStore {
    service: String,
}

impl KeychainSecretStore {
    pub fn new() -> Self {
        Self { service: KEYCHAIN_SERVICE.to_string() }
    }

    fn entry(&self, account: &str) -> Result<Entry> {
        Entry::new(&self.service, account).map_err(|failure| keychain_failure("open", account, &failure))
    }
}

impl Default for KeychainSecretStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SecretStore for KeychainSecretStore {
    fn put(&self, account: &str, secret: &Secret) -> Result<()> {
        self.entry(account)?.set_password(secret.expose()).map_err(|failure| keychain_failure("write", account, &failure))
    }

    fn read(&self, account: &str) -> Result<Secret> {
        match self.entry(account)?.get_password() {
            Ok(value) => Ok(Secret::new(value)),
            Err(keyring::Error::NoEntry) => Err(RekallError::not_found_msg(format!("No credential is stored under {account}."))),
            Err(failure) => Err(keychain_failure("read", account, &failure)),
        }
    }

    fn remove(&self, account: &str) -> Result<()> {
        match self.entry(account)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(failure) => Err(keychain_failure("remove", account, &failure)),
        }
    }
}

/// `keyring` describes the platform's refusal, never the value it was handed.
fn keychain_failure(action: &str, account: &str, failure: &keyring::Error) -> RekallError {
    error!(%failure, account, action, "The OS credential store refused a request");
    RekallError::internal(InternalFailure::Io, format!("The OS credential store could not {action} the credential of {account}: {failure}"))
}
