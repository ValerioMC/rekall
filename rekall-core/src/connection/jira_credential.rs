use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use rekall_kernel::{InternalFailure, RekallError, Result};

use super::{JiraDeployment, Secret};

/// What the store keeps in place of an email for a personal access token; it holds no `@`, so no email reads as it.
const PERSONAL_ACCESS_TOKEN: &str = "pat";

/// How Rekall signs in to a Jira site, which also says which Jira it is: Cloud takes the account's
/// email and an API token as Basic authentication, Data Center a personal access token as Bearer.
/// The store keeps `email:token` or `pat:token`, so an email holding `:` is refused.
#[derive(Debug)]
pub struct JiraCredential {
    email: Option<String>,
    token: Secret,
}

impl JiraCredential {
    pub fn cloud(email: &str, token: Secret) -> Result<Self> {
        let email = email.trim();
        if !email.contains('@') || email.contains(':') {
            return Err(RekallError::illegal("A Jira Cloud connection needs the email of the Atlassian account the API token belongs to."));
        }
        if token.is_blank() {
            return Err(RekallError::illegal("A Jira Cloud connection needs an API token of that account."));
        }
        Ok(Self { email: Some(email.to_string()), token })
    }

    pub fn data_center(token: Secret) -> Result<Self> {
        if token.is_blank() {
            return Err(RekallError::illegal("A Jira Data Center connection needs a personal access token."));
        }
        Ok(Self { email: None, token })
    }

    /// Reads back what `to_stored` kept.
    pub fn from_stored(stored: &Secret) -> Result<Self> {
        let corrupt = || RekallError::internal(InternalFailure::CorruptData, "a stored Jira credential is neither an email and a token nor a personal access token");
        let (account, token) = stored.expose().split_once(':').ok_or_else(corrupt)?;
        let token = Secret::new(token);
        match account {
            PERSONAL_ACCESS_TOKEN => Ok(Self { email: None, token }),
            email if email.contains('@') => Ok(Self { email: Some(email.to_string()), token }),
            _ => Err(corrupt()),
        }
    }

    pub fn to_stored(&self) -> Secret {
        Secret::new(format!("{}:{}", self.email.as_deref().unwrap_or(PERSONAL_ACCESS_TOKEN), self.token.expose()))
    }

    pub fn deployment(&self) -> JiraDeployment {
        if self.email.is_some() {
            JiraDeployment::Cloud
        } else {
            JiraDeployment::DataCenter
        }
    }

    /// Jira Cloud only: the account the API token belongs to.
    pub fn email(&self) -> Option<&str> {
        self.email.as_deref()
    }

    /// The `Authorization` header's value.
    pub fn authorization(&self) -> Secret {
        match &self.email {
            Some(email) => Secret::new(format!("Basic {}", STANDARD.encode(format!("{email}:{}", self.token.expose())))),
            None => Secret::new(format!("Bearer {}", self.token.expose())),
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/connection/jira_credential_tests.rs"]
mod tests;
