use rekall_kernel::Result;

use super::Secret;

/// Where connection credentials live, outside the database, keyed by a connection's `secret_ref`.
/// Calls block (a keychain may prompt), so async callers run them on a blocking thread. Never handed
/// to `rekall-mcp`: it is not part of `Services`.
pub trait SecretStore: Send + Sync {
    fn put(&self, account: &str, secret: &Secret) -> Result<()>;

    /// The credential under `account`, or `not_found` when none is stored.
    fn read(&self, account: &str) -> Result<Secret>;

    /// Removing an account that holds nothing succeeds, so a delete can always clean up.
    fn remove(&self, account: &str) -> Result<()>;
}
