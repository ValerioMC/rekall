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
//!   controls in `AnchorBar.vue` and calls these instead of a titlebar button;
//! - `installUpdate()` resolves to whether the newest release is being installed and restarted
//!   into (`update_install.rs`, outside this module because it stops the server the shell owns);
//! - `answerLeave(confirmed)` is the console's reply when the shell asks, in the console's own
//!   dialog, whether to quit or restart with a Claude session live (`exit_guard.rs`).
//!
//! A refusal rejects the promise with an `Error` whose message says why, as WebKit's reply
//! handler did.

// The command modules are public: `generate_handler!` resolves each command's hidden
// `__cmd__` companion by path, and a re-export does not carry it.
mod bridge_script;
mod claude_code_launch;
pub mod claude_code_launcher;
mod desktop_notice;
pub mod folder_picker;
mod home_path;
pub mod notifier;
pub mod window_controls;
mod splash_size;
mod window_fade;
mod window_geometry;

pub use bridge_script::bridge_script;
pub use claude_code_launch::ClaudeCodeLaunch;
pub use desktop_notice::DesktopNotice;
pub use window_controls::{allow_native_fullscreen, open_maximized};
pub use splash_size::splash_size;
pub use window_fade::{fade_in, fade_out, show_fading_in};
pub use window_geometry::WindowGeometry;

use home_path::expand_tilde;
