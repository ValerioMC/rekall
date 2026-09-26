use std::sync::Arc;

use rekall_common::Instant;

use crate::credentials::ClaudeCredentials;

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
