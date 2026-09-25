//! Reads the OAuth access token Claude Code keeps for the logged-in account: the macOS keychain
//! (`Claude Code-credentials`) first, then `~/.claude/.credentials.json`. Never writes or
//! refreshes. Every keychain item is read and the token with the latest `expiresAt` wins; one
//! whose `expiresAt` has passed is reported as absent rather than sent, since Anthropic would only
//! answer 401 and a stream of those is what gets this machine rate limited.

use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

use rekall_common::Instant;
use serde_json::Value;
use tracing::debug;

pub const KEYCHAIN_SERVICE: &str = "Claude Code-credentials";

/// Where the keychain half of the read comes from; the real one shells out to `security`.
pub trait KeychainSource: Send + Sync {
    fn secrets(&self) -> Vec<String>;
}

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

struct NoKeychain;

impl KeychainSource for NoKeychain {
    fn secrets(&self) -> Vec<String> {
        Vec::new()
    }
}

/// The macOS keychain through the `security` command. `find-generic-password` returns one item
/// per call and picks the first when several share a service, so the account names are listed
/// first with `dump-keychain` and each is read on its own.
struct SecurityCommandKeychain;

const COMMAND_TIMEOUT: Duration = Duration::from_secs(5);

impl KeychainSource for SecurityCommandKeychain {
    fn secrets(&self) -> Vec<String> {
        let accounts = accounts_holding_the_service();
        if accounts.is_empty() {
            return secret(None).into_iter().collect();
        }
        accounts.iter().filter_map(|account| secret(Some(account))).collect()
    }
}

fn accounts_holding_the_service() -> Vec<String> {
    let mut accounts: Vec<String> = Vec::new();
    let Some(dump) = run(&["security", "dump-keychain"]) else {
        return accounts;
    };
    let mut account: Option<String> = None;
    for line in dump.split('\n') {
        let trimmed = line.trim();
        if trimmed.starts_with("keychain:") {
            account = None;
        } else if trimmed.starts_with("\"acct\"<blob>=") {
            account = attribute_value(trimmed);
        } else if trimmed.starts_with("\"svce\"<blob>=") && attribute_value(trimmed).as_deref() == Some(KEYCHAIN_SERVICE) {
            if let Some(account) = &account {
                if !accounts.contains(account) {
                    accounts.push(account.clone());
                }
            }
        }
    }
    accounts
}

fn attribute_value(line: &str) -> Option<String> {
    let start = line.find("=\"")?;
    let end = line.rfind('"')?;
    if end <= start + 1 {
        return None;
    }
    Some(line[start + 2..end].to_string())
}

fn secret(account: Option<&str>) -> Option<String> {
    let mut command = vec!["security", "find-generic-password", "-s", KEYCHAIN_SERVICE];
    if let Some(account) = account {
        command.extend(["-a", account]);
    }
    command.push("-w");
    run(&command).map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

fn run(command: &[&str]) -> Option<String> {
    let mut child = Command::new(command[0])
        .args(&command[1..])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok()?;
    let deadline = std::time::Instant::now() + COMMAND_TIMEOUT;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let mut output = String::new();
                if let Some(mut stdout) = child.stdout.take() {
                    use std::io::Read;
                    let _ = stdout.read_to_string(&mut output);
                }
                return status.success().then_some(output);
            }
            Ok(None) if std::time::Instant::now() < deadline => std::thread::sleep(Duration::from_millis(20)),
            _ => {
                debug!("Timed out running {}", command[1]);
                let _ = child.kill();
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: &str = "2026-09-15T12:00:00Z";

    struct Secrets(Vec<String>);

    impl KeychainSource for Secrets {
        fn secrets(&self) -> Vec<String> {
            self.0.clone()
        }
    }

    fn now() -> Instant {
        Instant::parse(NOW).unwrap()
    }

    fn credentials(home: &std::path::Path, secrets: &[String]) -> ClaudeCredentials {
        let fixed = now();
        ClaudeCredentials::with(home.to_path_buf(), Arc::new(Secrets(secrets.to_vec())), Arc::new(move || fixed))
    }

    fn secret(token: &str, expires_at: Instant) -> String {
        format!("{{\"claudeAiOauth\":{{\"accessToken\":\"{token}\",\"expiresAt\":{}}}}}", expires_at.epoch_millis())
    }

    fn write_credentials(home: &std::path::Path, json: &str) {
        std::fs::create_dir_all(home.join(".claude")).unwrap();
        std::fs::write(home.join(".claude/.credentials.json"), json).unwrap();
    }

    #[test]
    fn reads_the_access_token_out_of_the_credentials_file() {
        let home = tempfile::tempdir().unwrap();
        write_credentials(home.path(), "{\"claudeAiOauth\":{\"accessToken\":\"sk-ant-oat01-abc\"}}");
        assert_eq!(credentials(home.path(), &[]).access_token().as_deref(), Some("sk-ant-oat01-abc"));
    }

    #[test]
    fn no_file_no_keychain_the_wrong_shape_or_a_blank_token_mean_no_token() {
        let home = tempfile::tempdir().unwrap();
        assert!(credentials(home.path(), &[]).access_token().is_none());
        write_credentials(home.path(), "{\"something\":\"else\"}");
        assert!(credentials(home.path(), &[]).access_token().is_none());
        write_credentials(home.path(), "{\"claudeAiOauth\":{\"accessToken\":\"  \"}}");
        assert!(credentials(home.path(), &[]).access_token().is_none());
    }

    #[test]
    fn of_several_keychain_items_the_one_refreshed_last_wins_whatever_its_order() {
        let home = tempfile::tempdir().unwrap();
        let stale = secret("stale", now().plus_seconds(-3600));
        let live = secret("live", now().plus_seconds(3600));
        assert_eq!(credentials(home.path(), &[stale.clone(), live.clone()]).access_token().as_deref(), Some("live"));
        assert_eq!(credentials(home.path(), &[live, stale]).access_token().as_deref(), Some("live"));
    }

    #[test]
    fn a_keychain_item_beats_the_credentials_file_when_it_expires_later() {
        let home = tempfile::tempdir().unwrap();
        write_credentials(home.path(), &secret("from-file", now().plus_seconds(60)));
        let keychain = secret("from-keychain", now().plus_seconds(120));
        assert_eq!(credentials(home.path(), &[keychain]).access_token().as_deref(), Some("from-keychain"));
    }

    #[test]
    fn a_token_past_its_expiry_is_absent() {
        let home = tempfile::tempdir().unwrap();
        assert!(credentials(home.path(), &[secret("expired", now().plus_seconds(-1))]).access_token().is_none());
    }

    #[test]
    fn a_token_with_no_expiry_is_kept_but_loses_to_one_that_says_when_it_expires() {
        let home = tempfile::tempdir().unwrap();
        let undated = "{\"claudeAiOauth\":{\"accessToken\":\"undated\"}}".to_string();
        assert_eq!(credentials(home.path(), std::slice::from_ref(&undated)).access_token().as_deref(), Some("undated"));
        let dated = secret("dated", now().plus_seconds(10));
        assert_eq!(credentials(home.path(), &[undated, dated]).access_token().as_deref(), Some("dated"));
    }

    #[test]
    fn an_unreadable_keychain_item_is_skipped_not_fatal() {
        let home = tempfile::tempdir().unwrap();
        let good = secret("good", now().plus_seconds(10));
        assert_eq!(credentials(home.path(), &["not json".into(), good]).access_token().as_deref(), Some("good"));
    }
}
