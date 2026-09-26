use std::path::PathBuf;
use std::sync::Arc;

use rekall_common::Instant;
use serde_json::Value;
use tracing::debug;

use super::KeychainSource;
use super::no_keychain::NoKeychain;
use super::security_command_keychain::SecurityCommandKeychain;

pub const KEYCHAIN_SERVICE: &str = "Claude Code-credentials";

/// One stored token and when Claude Code said it stops working.
#[derive(Clone, Debug)]
struct StoredToken {
    access_token: String,
    expires_at: Option<Instant>,
}

pub struct ClaudeCredentials {
    home: PathBuf,
    keychain: Arc<dyn KeychainSource>,
    clock: rekall_service::Clock,
}

impl ClaudeCredentials {
    pub fn new(home: PathBuf) -> Self {
        let keychain: Arc<dyn KeychainSource> =
            if cfg!(target_os = "macos") { Arc::new(SecurityCommandKeychain) } else { Arc::new(NoKeychain) };
        Self::with(home, keychain, rekall_service::system_clock())
    }

    pub fn with(home: PathBuf, keychain: Arc<dyn KeychainSource>, clock: rekall_service::Clock) -> Self {
        Self { home, keychain, clock }
    }

    /// The freshest token that is still valid, or `None` when every stored one is missing or expired.
    pub fn access_token(&self) -> Option<String> {
        let mut stored: Vec<StoredToken> = self.keychain.secrets().iter().filter_map(|s| token_from(s)).collect();
        if let Some(from_file) = self.read_credentials_file() {
            stored.push(from_file);
        }
        // `max` by expiry with nulls first: an undated token loses to any dated one; among equals
        // the first one listed stays, as `BinaryOperator.maxBy` kept the earlier of two equals.
        let mut freshest: Option<StoredToken> = None;
        for token in stored {
            let replace = match &freshest {
                None => true,
                Some(best) => token.expires_at > best.expires_at,
            };
            if replace {
                freshest = Some(token);
            }
        }
        let freshest = freshest?;
        if freshest.expires_at.is_some_and(|expiry| !expiry.is_after(&(self.clock)())) {
            debug!("Every stored Claude Code token has expired, the latest at {:?}", freshest.expires_at);
            return None;
        }
        Some(freshest.access_token)
    }

    fn read_credentials_file(&self) -> Option<StoredToken> {
        let path = self.home.join(".claude/.credentials.json");
        std::fs::read_to_string(path).ok().and_then(|json| token_from(&json))
    }
}

fn token_from(json: &str) -> Option<StoredToken> {
    if json.trim().is_empty() {
        return None;
    }
    let root: Value = serde_json::from_str(json).ok()?;
    let oauth = root.get("claudeAiOauth")?;
    let value = oauth.get("accessToken").and_then(Value::as_str).unwrap_or("").trim().to_string();
    if value.is_empty() {
        return None;
    }
    let expires_at = oauth.get("expiresAt").and_then(Value::as_i64).and_then(Instant::from_epoch_millis);
    Some(StoredToken { access_token: value, expires_at })
}
