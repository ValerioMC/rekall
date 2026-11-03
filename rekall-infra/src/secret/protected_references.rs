use std::path::Path;

use crate::connection::CREDENTIAL_ENDPOINT_PREFIX;
use crate::credentials::CREDENTIALS_FILE;

use super::SECRETS_FILE;

/// What a session must never read, copy or search: Rekall's credential file, Claude Code's and the
/// URLs the credential helpers ask. A tool call that names any of them is refused by the guard.
pub fn protected_references(rekall_home: &Path, user_home: &Path) -> Vec<String> {
    vec![
        rekall_home.join(SECRETS_FILE).display().to_string(),
        user_home.join(CREDENTIALS_FILE).display().to_string(),
        CREDENTIAL_ENDPOINT_PREFIX.to_string(),
    ]
}

#[cfg(test)]
#[path = "../../tests/unit/secret/protected_references_tests.rs"]
mod tests;
