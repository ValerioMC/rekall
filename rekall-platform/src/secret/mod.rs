//! Where connection credentials are kept: the operating system's own credential store.

mod keychain_secret_store;

pub use keychain_secret_store::KeychainSecretStore;
