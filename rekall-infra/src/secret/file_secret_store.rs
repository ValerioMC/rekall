use std::collections::BTreeMap;
use std::fs;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use rekall_core::connection::{Secret, SecretStore};
use rekall_kernel::{InternalFailure, RekallError, Result};
use tempfile::NamedTempFile;
use tracing::error;

/// Under `rekall.home`.
pub const SECRETS_FILE: &str = "secrets.json";

/// Rekall's own credential file: one JSON object of account to value, readable by its owner only.
/// Nothing is encrypted, so whoever reads the file reads every credential; the file stays out of the
/// database and its backups.
pub struct FileSecretStore {
    path: PathBuf,
    writing: Mutex<()>,
}

impl FileSecretStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path, writing: Mutex::new(()) }
    }

    fn lock(&self) -> Result<MutexGuard<'_, ()>> {
        self.writing.lock().map_err(|_| RekallError::internal(InternalFailure::Background, "the credential file lock was poisoned"))
    }

    fn load(&self) -> Result<BTreeMap<String, String>> {
        match fs::read_to_string(&self.path) {
            Ok(text) => serde_json::from_str(&text).map_err(|failure| file_failure("parse", &self.path, &failure)),
            Err(failure) if failure.kind() == ErrorKind::NotFound => Ok(BTreeMap::new()),
            Err(failure) => Err(file_failure("read", &self.path, &failure)),
        }
    }

    fn save(&self, entries: &BTreeMap<String, String>) -> Result<()> {
        let folder = self.path.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(folder).map_err(|failure| file_failure("create the folder of", &self.path, &failure))?;
        let mut file = NamedTempFile::new_in(folder).map_err(|failure| file_failure("create", &self.path, &failure))?;
        serde_json::to_writer(&mut file, entries).map_err(|failure| file_failure("write", &self.path, &failure))?;
        file.flush().map_err(|failure| file_failure("write", &self.path, &failure))?;
        file.persist(&self.path).map_err(|failure| file_failure("replace", &self.path, &failure.error))?;
        Ok(())
    }
}

impl SecretStore for FileSecretStore {
    fn put(&self, account: &str, secret: &Secret) -> Result<()> {
        let _writing = self.lock()?;
        let mut entries = self.load()?;
        entries.insert(account.to_string(), secret.expose().to_string());
        self.save(&entries)
    }

    fn read(&self, account: &str) -> Result<Secret> {
        let _writing = self.lock()?;
        self.load()?
            .remove(account)
            .map(Secret::new)
            .ok_or_else(|| RekallError::not_found_msg(format!("No credential is stored under {account}.")))
    }

    fn remove(&self, account: &str) -> Result<()> {
        let _writing = self.lock()?;
        let mut entries = self.load()?;
        if entries.remove(account).is_none() {
            return Ok(());
        }
        self.save(&entries)
    }
}

/// Names the file and the failure, never a value read from it.
fn file_failure(action: &str, path: &Path, failure: &dyn std::fmt::Display) -> RekallError {
    error!(%failure, path = %path.display(), action, "The credential file refused a request");
    RekallError::internal(InternalFailure::Io, format!("Rekall could not {action} its credential file {}: {failure}", path.display()))
}

#[cfg(test)]
#[path = "../../tests/unit/secret/file_secret_store_tests.rs"]
mod tests;
