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
mod window_geometry;

pub use bridge_script::BRIDGE;
pub use claude_code_launch::ClaudeCodeLaunch;
pub use desktop_notice::DesktopNotice;
pub use window_controls::{allow_native_fullscreen, open_maximized};
pub use window_geometry::WindowGeometry;

use home_path::expand_tilde;
