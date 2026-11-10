use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use rekall_kernel::{RekallError, Result};

use crate::connection::Secret;

/// How Rekall signs in to a Grafana instance: a service account token as Bearer, or a user and
/// password as Basic. The store keeps the bare token or `user:password`, so a token or a user
/// holding `:` is refused; a password may hold one.
#[derive(Debug)]
pub struct GrafanaCredential {
    user: Option<String>,
    secret: Secret,
}

impl GrafanaCredential {
    pub fn service_account_token(token: Secret) -> Result<Self> {
        if token.is_blank() {
            return Err(RekallError::illegal("A Grafana connection needs its service account token, or a user and a password."));
        }
        if token.expose().contains(':') {
            return Err(RekallError::illegal("A Grafana service account token holds no ':'. To sign in with a password, type the user too."));
        }
        Ok(Self { user: None, secret: token })
    }

    pub fn password(user: &str, password: Secret) -> Result<Self> {
        let user = user.trim();
        if user.is_empty() || user.contains(':') {
            return Err(RekallError::illegal("A Grafana user name is not blank and holds no ':'."));
        }
        if password.is_blank() {
            return Err(RekallError::illegal("A Grafana connection signed in with a user needs that user's password."));
        }
        Ok(Self { user: Some(user.to_string()), secret: password })
    }

    /// Reads back what `to_stored` kept.
    pub fn from_stored(stored: &Secret) -> Result<Self> {
        match stored.expose().split_once(':') {
            Some((user, password)) => Self::password(user, Secret::new(password)),
            None => Self::service_account_token(Secret::new(stored.expose())),
        }
    }

    pub fn to_stored(&self) -> Secret {
        match &self.user {
            Some(user) => Secret::new(format!("{user}:{}", self.secret.expose())),
            None => Secret::new(self.secret.expose()),
        }
    }

    /// The user name, when the credential is a password.
    pub fn user(&self) -> Option<&str> {
        self.user.as_deref()
    }

    /// The service account token, or the password.
    pub fn secret(&self) -> &Secret {
        &self.secret
    }

    /// The `Authorization` header's value.
    pub fn authorization(&self) -> Secret {
        match &self.user {
            Some(user) => Secret::new(format!("Basic {}", STANDARD.encode(format!("{user}:{}", self.secret.expose())))),
            None => Secret::new(format!("Bearer {}", self.secret.expose())),
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/connection/model/grafana_credential_tests.rs"]
mod tests;
