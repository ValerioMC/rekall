use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use rekall_common::{InternalFailure, RekallError, Result};

use super::Secret;

/// A Jira Cloud credential: the account's email and one of its API tokens. The store keeps them as
/// the `email:token` pair Basic authentication sends, so an email holding `:` is refused.
#[derive(Debug)]
pub struct JiraCredential {
    email: String,
    token: Secret,
}

impl JiraCredential {
    pub fn new(email: &str, token: Secret) -> Result<Self> {
        let email = email.trim();
        if !email.contains('@') || email.contains(':') {
            return Err(RekallError::illegal("A Jira connection needs the email of the Atlassian account the API token belongs to."));
        }
        if token.is_blank() {
            return Err(RekallError::illegal("A Jira connection needs an API token of that account."));
        }
        Ok(Self { email: email.to_string(), token })
    }

    /// Reads back what `to_stored` kept.
    pub fn from_stored(stored: &Secret) -> Result<Self> {
        let (email, token) = stored
            .expose()
            .split_once(':')
            .ok_or_else(|| RekallError::internal(InternalFailure::CorruptData, "a stored Jira credential is not an email and a token"))?;
        Ok(Self { email: email.to_string(), token: Secret::new(token) })
    }

    pub fn to_stored(&self) -> Secret {
        Secret::new(format!("{}:{}", self.email, self.token.expose()))
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    /// The `Authorization` header's value.
    pub fn authorization(&self) -> Secret {
        Secret::new(format!("Basic {}", STANDARD.encode(self.to_stored().expose())))
    }
}

#[cfg(test)]
#[path = "../../tests/unit/connection/jira_credential_tests.rs"]
mod tests;
