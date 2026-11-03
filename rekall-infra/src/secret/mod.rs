//! Where credentials are kept: Rekall's own file, and the operating system's credential store they
//! were kept in before.

mod file_secret_store;
mod keychain_secret_store;

pub use file_secret_store::FileSecretStore;
pub use keychain_secret_store::KeychainSecretStore;
