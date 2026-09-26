use std::process::Command;
use std::time::Duration;

use tracing::debug;

use super::{KEYCHAIN_SERVICE, KeychainSource};

/// The macOS keychain through the `security` command. `find-generic-password` returns one item
/// per call and picks the first when several share a service, so the account names are listed
/// first with `dump-keychain` and each is read on its own.
pub(super) struct SecurityCommandKeychain;

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

/// A child whose stdout is not drained as it runs deadlocks once its output outgrows the OS pipe
/// buffer (`dump-keychain` on a real keychain is well past that). The read has to run on its own
/// thread, concurrently with the wait, not after it.
fn run(command: &[&str]) -> Option<String> {
    let mut child = Command::new(command[0])
        .args(&command[1..])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok()?;
    let mut stdout = child.stdout.take()?;
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        use std::io::Read;
        let mut output = String::new();
        let _ = stdout.read_to_string(&mut output);
        let _ = sender.send(output);
    });
    let deadline = std::time::Instant::now() + COMMAND_TIMEOUT;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let output = receiver.recv_timeout(Duration::from_millis(500)).unwrap_or_default();
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
#[path = "security_command_keychain_tests.rs"]
mod tests;
