use std::path::{Path, PathBuf};

use super::{expand_tilde, ClaudeCodeLaunch};

/// What an anchor may contain: what `entity:value` needs and nothing that means anything to a
/// shell. Anything else is refused rather than escaped.
const ANCHOR_CHARACTERS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789:._- ";
const ANCHOR_LIMIT: usize = 300;
const HOME_RELATIVE_BINARIES: [&str; 3] = [".local/bin/claude", ".claude/local/claude", "bin/claude"];
const KNOWN_DIRECTORIES: [&str; 3] = ["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin"];

/// Opens a terminal already loading a working context. The page never names a command: it sends
/// a folder, an anchor and one flag, each checked here before a line of shell is written.
#[tauri::command]
pub async fn open_in_claude_code(launch: ClaudeCodeLaunch) -> Result<String, String> {
    let directory = expand_tilde(&launch.directory);
    if launch.directory.is_empty() || !directory.is_dir() {
        let named = if launch.directory.is_empty() { "No folder" } else { launch.directory.as_str() };
        return Err(format!("{named} is not a folder on this machine"));
    }
    if !is_safe_anchor(&launch.anchors) {
        return Err("That anchor has characters an anchor cannot have".into());
    }
    let home = dirs::home_dir().unwrap_or_default();
    let script = script(&directory.to_string_lossy(), &launch.anchors, launch.skip_permissions, &claude_binary(&home));
    let file = write_script(&script).map_err(|e| e.to_string())?;
    open_terminal(&file)
}

pub fn is_safe_anchor(anchors: &str) -> bool {
    let trimmed = anchors.trim_matches([' ', '\t']);
    !trimmed.is_empty() && trimmed.chars().count() <= ANCHOR_LIMIT && trimmed.chars().all(|c| ANCHOR_CHARACTERS.contains(c))
}

/// The whole session in four lines. It removes itself first (the shell already holds the file
/// open), and `exec` puts Claude Code in the shell's place, so closing the window ends it.
pub fn script(directory: &str, anchors: &str, skip_permissions: bool, claude: &str) -> String {
    let flag = if skip_permissions { " --dangerously-skip-permissions" } else { "" };
    let command = format!("{}{flag} {}", quote(claude), quote(&format!("/rk {}", anchors.trim_matches([' ', '\t']))));
    format!("#!/bin/sh\nrm -f \"$0\"\ncd {} || exit 1\nexec {command}\n", quote(directory))
}

/// POSIX single quoting: everything inside is literal, and only the quote itself needs a way out.
pub fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

/// Where Claude Code installs itself, then the usual directories, then the bare name for a login
/// shell's PATH to resolve.
pub fn claude_binary(home: &Path) -> String {
    for candidate in HOME_RELATIVE_BINARIES {
        let path = home.join(candidate);
        if is_executable(&path) {
            return path.to_string_lossy().into_owned();
        }
    }
    for directory in KNOWN_DIRECTORIES {
        let path = Path::new(directory).join("claude");
        if is_executable(&path) {
            return path.to_string_lossy().into_owned();
        }
    }
    "claude".into()
}

fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        path.metadata().is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
    }
    #[cfg(not(unix))]
    {
        path.is_file()
    }
}

fn write_script(script: &str) -> std::io::Result<PathBuf> {
    let file = std::env::temp_dir().join(format!("rekall-{}.command", uuid::Uuid::new_v4().to_string().to_uppercase()));
    std::fs::write(&file, script)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(file)
}

/// iTerm when it is installed, Terminal otherwise: the script is opened with the terminal rather
/// than the terminal scripted, so no "Rekall wants to control Terminal" prompt stands in the way.
#[cfg(target_os = "macos")]
fn open_terminal(file: &Path) -> Result<String, String> {
    use std::process::Command;
    for (bundle, name) in [("com.googlecode.iterm2", "iTerm"), ("com.apple.Terminal", "Terminal")] {
        let opened = Command::new("open").args(["-b", bundle]).arg(file).status();
        if opened.is_ok_and(|status| status.success()) {
            return Ok(name.into());
        }
    }
    Err("Neither iTerm nor Terminal could open the session".into())
}

/// Not in the macOS launcher, which only ran on macOS: the terminal the desktop names as its
/// default, then the common ones.
#[cfg(all(unix, not(target_os = "macos")))]
fn open_terminal(file: &Path) -> Result<String, String> {
    use std::process::Command;
    let candidates: [(&str, &[&str]); 4] =
        [("x-terminal-emulator", &["-e"]), ("gnome-terminal", &["--"]), ("konsole", &["-e"]), ("xterm", &["-e"])];
    for (program, before) in candidates {
        if Command::new(program).args(before).arg(file).spawn().is_ok() {
            return Ok(program.into());
        }
    }
    Err("No terminal emulator was found to open the session in".into())
}

#[cfg(not(unix))]
fn open_terminal(_file: &Path) -> Result<String, String> {
    Err("Opening a terminal from Rekall is only available on macOS and Linux".into())
}

#[cfg(test)]
#[path = "../../tests/unit/bridges/claude_code_launcher_tests.rs"]
mod tests;
