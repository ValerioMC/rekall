//! The environment a new window of the user's own terminal would have. An app opened from Finder
//! inherits launchd's bare `PATH`, so anything a shell profile adds is missing from it. This asks
//! the login shell itself: `$SHELL -i -l -c` prints its environment between two markers, which
//! keeps whatever the profile echoes out of it.
//!
//! Resolved in the background and cached. Every read hands back the cached copy and starts a
//! refresh, so a tool installed while Rekall runs reaches the next terminal but one without a
//! restart. A shell that fails, prints no markers or outlives the timeout leaves the last good
//! copy in place, or this process's own environment when there is none. Windows has no login
//! shell to ask and gets the process environment.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use tokio::sync::watch;
use tracing::warn;

/// Set while resolving, so a profile can skip slow work.
pub const RESOLVING_FLAG: &str = "REKALL_RESOLVING_ENVIRONMENT";

/// Variables that describe the resolving shell itself, not the user's setup.
const SHELL_SESSION_VARIABLES: [&str; 5] = ["_", "SHLVL", "PWD", "OLDPWD", RESOLVING_FLAG];

pub type Environment = HashMap<String, String>;

pub struct LoginShellEnvironment {
    shell: Option<PathBuf>,
    timeout: Duration,
    latest: RwLock<Option<Environment>>,
    inflight: Mutex<Option<watch::Receiver<Option<Environment>>>>,
}

impl LoginShellEnvironment {
    pub fn new(shell_override: Option<&str>, timeout: Duration) -> Arc<Self> {
        Arc::new(Self {
            shell: choose_shell(shell_override),
            timeout,
            latest: RwLock::new(None),
            inflight: Mutex::new(None),
        })
    }

    /// Start the first resolution as the application comes up.
    pub fn warm(self: &Arc<Self>) {
        let _ = self.refresh();
    }

    /// The cached login-shell environment, waiting for the first resolution if none has finished.
    pub async fn current(self: &Arc<Self>) -> Environment {
        let known = self.latest.read().ok().and_then(|l| l.clone());
        let mut pending = self.refresh();
        if let Some(known) = known {
            return known;
        }
        let resolved = match pending.wait_for(Option::is_some).await {
            Ok(value) => value.clone(),
            Err(_) => None,
        };
        resolved.unwrap_or_else(process_environment)
    }

    /// Start a resolution unless one is already running, and return the one that is.
    fn refresh(self: &Arc<Self>) -> watch::Receiver<Option<Environment>> {
        let mut inflight = self.inflight.lock().expect("the lock is never poisoned");
        if let Some(running) = inflight.as_ref() {
            return running.clone();
        }
        let (sender, receiver) = watch::channel(None);
        *inflight = Some(receiver.clone());
        drop(inflight);
        let this = self.clone();
        tokio::task::spawn_blocking(move || {
            if let Some(resolved) = this.resolve() {
                if let Ok(mut latest) = this.latest.write() {
                    *latest = Some(resolved);
                }
            }
            let known = this.latest.read().ok().and_then(|l| l.clone());
            if let Ok(mut inflight) = this.inflight.lock() {
                *inflight = None;
            }
            let _ = sender.send(Some(known.unwrap_or_else(process_environment)));
        });
        receiver
    }

    /// One run of the login shell. `None` on any failure, which is logged with its reason.
    pub fn resolve(&self) -> Option<Environment> {
        let shell = self.shell.as_ref()?;
        let marker = format!("rekall-{}", rekall_common::Id::random());
        let script = format!("printf '%s' '{marker}'; /usr/bin/env -0; printf '%s' '{marker}'");
        let output = tempfile::NamedTempFile::with_prefix("rekall-shell-environment").ok()?;
        let stdout = output.reopen().ok()?;
        let mut command = Command::new(shell);
        command
            .args(["-i", "-l", "-c", &script])
            .stdin(Stdio::null())
            .stdout(stdout)
            .stderr(Stdio::null())
            .env(RESOLVING_FLAG, "1");
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(failed) => {
                warn!("Could not read the environment of {}: {failed}", shell.display());
                return None;
            }
        };
        let deadline = Instant::now() + self.timeout;
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(20)),
                Ok(None) => {
                    warn!(
                        "{} took longer than {}s to print its environment; terminals keep the last one known",
                        shell.display(),
                        self.timeout.as_secs()
                    );
                    kill_group(&mut child);
                    return None;
                }
                Err(_) => {
                    kill_group(&mut child);
                    return None;
                }
            }
        };
        let text = std::fs::read(output.path()).map(|b| String::from_utf8_lossy(&b).into_owned()).unwrap_or_default();
        let environment = parse(&text, &marker);
        if environment.is_none() {
            warn!(
                "{} exited with code {} without printing its environment",
                shell.display(),
                status.code().unwrap_or(-1)
            );
        }
        environment
    }
}

fn kill_group(child: &mut std::process::Child) {
    #[cfg(unix)]
    unsafe {
        libc::kill(-(child.id() as i32), libc::SIGKILL);
    }
    let _ = child.kill();
    let _ = child.wait();
}

/// The `env -0` block between the two markers, minus the resolving shell's own variables.
pub fn parse(output: &str, marker: &str) -> Option<Environment> {
    let start = output.find(marker)?;
    let end = output.rfind(marker)?;
    if end <= start {
        return None;
    }
    let mut environment = HashMap::new();
    for entry in output[start + marker.len()..end].split('\0') {
        let Some(equals) = entry.find('=') else { continue };
        if equals == 0 {
            continue;
        }
        let key = &entry[..equals];
        if !SHELL_SESSION_VARIABLES.contains(&key) {
            environment.insert(key.to_string(), entry[equals + 1..].to_string());
        }
    }
    if environment.is_empty() {
        None
    } else {
        Some(environment)
    }
}

pub fn process_environment() -> Environment {
    std::env::vars().collect()
}

fn choose_shell(shell_override: Option<&str>) -> Option<PathBuf> {
    if cfg!(windows) {
        return None;
    }
    let configured = match shell_override.map(str::trim).filter(|s| !s.is_empty()) {
        Some(shell) => Some(shell.to_string()),
        None => std::env::var("SHELL").ok().filter(|s| !s.trim().is_empty()),
    };
    if let Some(configured) = configured {
        return Some(PathBuf::from(configured));
    }
    let zsh = PathBuf::from("/bin/zsh");
    Some(if is_executable(&zsh) { zsh } else { PathBuf::from("/bin/sh") })
}

pub fn is_executable(path: &std::path::Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path).map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0).unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        path.is_file()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_shell(dir: &std::path::Path, body: &str) -> String {
        let shell = dir.join("fake-shell");
        std::fs::write(&shell, format!("#!/bin/sh\n{body}\neval \"$4\"\n")).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&shell, std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        shell.to_string_lossy().into_owned()
    }

    fn resolve(shell: &str, timeout: u64) -> Option<Environment> {
        LoginShellEnvironment::new(Some(shell), Duration::from_secs(timeout)).resolve()
    }

    #[test]
    fn what_the_profile_exports_comes_back_and_what_it_prints_around_the_markers_does_not() {
        let dir = tempfile::tempdir().unwrap();
        let shell = fake_shell(dir.path(), "export PATH=\"/home/me/.cargo/bin:$PATH\"\nexport RUSTUP_HOME=/home/me/.rustup\necho 'Welcome back, last login yesterday'");
        let environment = resolve(&shell, 5).unwrap();
        assert!(environment["PATH"].starts_with("/home/me/.cargo/bin:"));
        assert_eq!(environment["RUSTUP_HOME"], "/home/me/.rustup");
        assert!(!environment.keys().any(|k| k.contains("Welcome")));
    }

    #[test]
    fn the_resolving_shells_own_variables_and_the_flag_are_left_out() {
        let dir = tempfile::tempdir().unwrap();
        let shell = fake_shell(dir.path(), "[ -n \"$REKALL_RESOLVING_ENVIRONMENT\" ] && export SAW_FLAG=yes");
        let environment = resolve(&shell, 5).unwrap();
        assert_eq!(environment["SAW_FLAG"], "yes");
        for key in ["_", "SHLVL", "PWD", "OLDPWD", RESOLVING_FLAG] {
            assert!(!environment.contains_key(key), "{key}");
        }
    }

    #[test]
    fn a_value_holding_an_equals_sign_or_a_newline_survives_intact() {
        let dir = tempfile::tempdir().unwrap();
        let shell = fake_shell(dir.path(), "export ODD='a=b\nsecond line'");
        assert_eq!(resolve(&shell, 5).unwrap()["ODD"], "a=b\nsecond line");
    }

    #[test]
    fn a_shell_that_never_finishes_is_abandoned_at_the_timeout() {
        let dir = tempfile::tempdir().unwrap();
        let shell = fake_shell(dir.path(), "sleep 30");
        let started = Instant::now();
        assert!(resolve(&shell, 1).is_none());
        assert!(started.elapsed() < Duration::from_secs(10));
    }

    #[test]
    fn a_shell_that_exits_before_printing_or_is_not_there_gives_nothing() {
        let dir = tempfile::tempdir().unwrap();
        assert!(resolve(&fake_shell(dir.path(), "exit 1"), 5).is_none());
        assert!(resolve(&dir.path().join("missing").to_string_lossy(), 5).is_none());
    }

    #[tokio::test]
    async fn with_no_answer_current_falls_back_to_this_processs_environment() {
        let dir = tempfile::tempdir().unwrap();
        let login = LoginShellEnvironment::new(Some(&fake_shell(dir.path(), "exit 1")), Duration::from_secs(5));
        assert_eq!(login.current().await, process_environment());
    }

    #[tokio::test]
    async fn current_hands_back_the_resolved_environment() {
        let dir = tempfile::tempdir().unwrap();
        let login = LoginShellEnvironment::new(Some(&fake_shell(dir.path(), "export FROM_PROFILE=1")), Duration::from_secs(5));
        assert_eq!(login.current().await["FROM_PROFILE"], "1");
    }

    #[test]
    fn parse_needs_both_markers() {
        assert!(parse("noise only", "MARK").is_none());
        assert!(parse("MARKA=1\0", "MARK").is_none());
        let parsed = parse("noise MARKA=1\0B=2\0MARK noise", "MARK").unwrap();
        assert_eq!(parsed, HashMap::from([("A".to_string(), "1".to_string()), ("B".to_string(), "2".to_string())]));
    }
}
