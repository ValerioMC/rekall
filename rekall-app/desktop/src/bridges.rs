//! What the window can do that a browser tab cannot, offered to the page as `window.rekallDesktop`
//! with the call shape the macOS launcher's WKWebView bridges had:
//!
//! - `pickFolder(currentPath)` resolves to an absolute path, or null when the dialog is cancelled
//!   (`FolderPicker`);
//! - `openInClaudeCode({directory, anchors, skipPermissions})` resolves to the name of the terminal
//!   it opened (`ClaudeCodeLauncher`);
//! - `notify({title, body})` resolves to whether the system showed it (`Notifier`);
//! - `closeWindow()`, `minimizeWindow()` and `toggleMaximizeWindow()` stand in for the native
//!   traffic lights the window has none of: the console draws its own close/minimize/maximize
//!   controls in `AnchorBar.vue` and calls these instead of a titlebar button.
//!
//! A refusal rejects the promise with an `Error` whose message says why, as WebKit's reply
//! handler did.

use std::path::{Path, PathBuf};

use serde::Deserialize;
use tauri::{AppHandle, Runtime, WebviewWindow};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_notification::{NotificationExt, PermissionState};

/// The JavaScript half, installed before the page runs. It merges rather than assigns, as the
/// three Swift bridges did, and turns a rejected invoke into an `Error` the console can read.
pub const BRIDGE: &str = r#"
(function () {
  if (!window.__TAURI_INTERNALS__) return;
  var call = function (command, args) {
    return window.__TAURI_INTERNALS__.invoke(command, args).catch(function (failure) {
      throw new Error(typeof failure === 'string' ? failure : (failure && failure.message) || String(failure));
    });
  };
  window.rekallDesktop = Object.assign(window.rekallDesktop || {}, {
    pickFolder: function (currentPath) {
      return call('pick_folder', { path: typeof currentPath === 'string' ? currentPath : '' });
    },
    openInClaudeCode: function (launch) {
      return call('open_in_claude_code', { launch: {
        directory: String((launch && launch.directory) || ''),
        anchors: String((launch && launch.anchors) || ''),
        skipPermissions: Boolean(launch && launch.skipPermissions)
      } });
    },
    notify: function (notice) {
      return call('notify', { notice: {
        title: String((notice && notice.title) || ''),
        body: String((notice && notice.body) || '')
      } });
    },
    closeWindow: function () {
      return call('close_window', {});
    },
    minimizeWindow: function () {
      return call('minimize_window', {});
    },
    toggleMaximizeWindow: function () {
      return call('toggle_maximize_window', {});
    }
  });
})();
"#;

// ---------------------------------------------------------------- the folder chooser

/// A real folder dialog, attached to the window: the path it produces is the same absolute path
/// the server is about to validate. It may create the folder, since the server never does.
#[tauri::command]
pub async fn pick_folder<R: Runtime>(window: WebviewWindow<R>, path: Option<String>) -> Result<Option<String>, String> {
    let (answer, chosen) = tokio::sync::oneshot::channel();
    let mut dialog = window
        .dialog()
        .file()
        .set_title("Choose the folder that holds the Rekall database")
        .set_can_create_directories(true)
        .set_parent(&window);
    if let Some(typed) = path.filter(|p| !p.is_empty()) {
        let expanded = expand_tilde(&typed);
        if expanded.is_dir() {
            dialog = dialog.set_directory(expanded);
        }
    }
    dialog.pick_folder(move |folder| {
        let _ = answer.send(folder.and_then(|f| f.into_path().ok()).map(|p| p.to_string_lossy().into_owned()));
    });
    chosen.await.map_err(|_| "The folder dialog closed without an answer".to_string())
}

// ---------------------------------------------------------------- the terminal

/// What an anchor may contain: what `entity:value` needs and nothing that means anything to a
/// shell. Anything else is refused rather than escaped.
const ANCHOR_CHARACTERS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789:._- ";
const ANCHOR_LIMIT: usize = 300;
const HOME_RELATIVE_BINARIES: [&str; 3] = [".local/bin/claude", ".claude/local/claude", "bin/claude"];
const KNOWN_DIRECTORIES: [&str; 3] = ["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin"];

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCodeLaunch {
    #[serde(default)]
    directory: String,
    #[serde(default)]
    anchors: String,
    #[serde(default)]
    skip_permissions: bool,
}

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

// ---------------------------------------------------------------- notifications

const TITLE_LIMIT: usize = 120;
const BODY_LIMIT: usize = 300;

#[derive(Debug, Deserialize)]
pub struct DesktopNotice {
    #[serde(default)]
    title: String,
    #[serde(default)]
    body: String,
}

/// A banner for a claim that arrived while the console was not looked at. Title and body only,
/// cut to what a banner shows; a refused permission resolves to false rather than failing.
#[tauri::command]
pub async fn notify<R: Runtime>(app: AppHandle<R>, notice: DesktopNotice) -> Result<bool, String> {
    if notice.title.is_empty() {
        return Err("A notification needs a title".into());
    }
    let notifications = app.notification();
    let mut permission = notifications.permission_state().unwrap_or(PermissionState::Denied);
    if permission != PermissionState::Granted {
        permission = notifications.request_permission().unwrap_or(PermissionState::Denied);
    }
    if permission != PermissionState::Granted {
        return Ok(false);
    }
    let title: String = notice.title.chars().take(TITLE_LIMIT).collect();
    let body: String = notice.body.chars().take(BODY_LIMIT).collect();
    Ok(notifications.builder().title(title).body(body).show().is_ok())
}

// ---------------------------------------------------------------- window controls

/// The window has no native close/minimize/maximize buttons (`decorations(false)` in `main.rs`);
/// these stand in for them, called from the controls `AnchorBar.vue` draws in the console's own
/// header.
#[tauri::command]
pub fn close_window<R: Runtime>(window: WebviewWindow<R>) -> Result<(), String> {
    window.close().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn minimize_window<R: Runtime>(window: WebviewWindow<R>) -> Result<(), String> {
    window.minimize().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_maximize_window<R: Runtime>(window: WebviewWindow<R>) -> Result<(), String> {
    let maximized = window.is_maximized().map_err(|e| e.to_string())?;
    if maximized { window.unmaximize() } else { window.maximize() }.map_err(|e| e.to_string())
}

fn expand_tilde(path: &str) -> PathBuf {
    match path.strip_prefix('~') {
        Some(rest) if rest.is_empty() || rest.starts_with('/') => {
            PathBuf::from(format!("{}{rest}", dirs::home_dir().unwrap_or_default().to_string_lossy()))
        }
        _ => PathBuf::from(path),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_anchor_is_letters_digits_and_the_few_marks_an_anchor_uses() {
        assert!(is_safe_anchor("project:vega task:report-builder"));
        assert!(is_safe_anchor("  task:a.b_c-d  "));
        assert!(!is_safe_anchor("   "));
        assert!(!is_safe_anchor("task:x; rm -rf ~"));
        assert!(!is_safe_anchor("task:$(whoami)"));
        assert!(!is_safe_anchor("task:'quoted'"));
        assert!(!is_safe_anchor(&"a".repeat(301)));
        assert!(is_safe_anchor(&"a".repeat(300)));
    }

    #[test]
    fn the_script_changes_into_the_folder_and_execs_claude_on_the_anchor() {
        assert_eq!(
            script("/Users/me/Projects/it's here", "project:vega task:x ", true, "/Users/me/.local/bin/claude"),
            "#!/bin/sh\nrm -f \"$0\"\ncd '/Users/me/Projects/it'\\''s here' || exit 1\n\
             exec '/Users/me/.local/bin/claude' --dangerously-skip-permissions '/rk project:vega task:x'\n"
        );
        assert!(!script("/p", "task:x", false, "claude").contains("--dangerously-skip-permissions"));
    }

    #[test]
    fn claude_is_looked_for_where_it_installs_itself_first() {
        let home = tempfile_home();
        let installed = home.join(".claude/local/claude");
        std::fs::create_dir_all(installed.parent().unwrap()).unwrap();
        std::fs::write(&installed, "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&installed, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        assert_eq!(claude_binary(&home), installed.to_string_lossy());
        std::fs::remove_dir_all(&home).unwrap();
    }

    fn tempfile_home() -> PathBuf {
        let home = std::env::temp_dir().join(format!("rekall-desktop-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&home).unwrap();
        home
    }
}
