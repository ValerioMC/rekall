//! Where credentials are kept: Rekall's own file, and the operating system's credential store they
//! were kept in before.

mod file_secret_store;
mod keychain_secret_store;
mod protected_references;

pub use file_secret_store::{FileSecretStore, SECRETS_FILE};
pub use keychain_secret_store::KeychainSecretStore;
pub use protected_references::protected_references;
