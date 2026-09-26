use tauri::{PhysicalRect, Runtime, State, WebviewWindow};

use super::WindowGeometry;

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

/// Grows `window` to its monitor's work area and remembers its current size to restore later.
/// Called once at startup (`main.rs`) so the window opens filling the screen, the same way
/// `toggle_maximize_window` grows it from the console's own maximize button. Returns that work
/// area, so a caller showing a still-hidden window can wait for the (asynchronously applied,
/// and not necessarily applied to position and size in the same instant) resize to actually
/// land there first.
pub fn open_maximized<R: Runtime>(window: &WebviewWindow<R>, geometry: &WindowGeometry) -> Result<PhysicalRect<i32, u32>, String> {
    let restore = (window.outer_position().map_err(|e| e.to_string())?, window.inner_size().map_err(|e| e.to_string())?);
    let area = work_area(window)?;
    geometry.grow(window, area, restore)?;
    Ok(area)
}

#[tauri::command]
pub fn toggle_maximize_window<R: Runtime>(window: WebviewWindow<R>, geometry: State<'_, WindowGeometry>) -> Result<(), String> {
    // Bound to its own statement rather than matched on directly: a match scrutinee's temporaries
    // live until the end of the match, so matching on the lock guard inline would still hold it
    // while the `None` arm calls back into `grow`, which locks the same `Mutex` again — a
    // non-reentrant self-deadlock that froze the app on every other click of the console's own
    // maximize button.
    let stored = geometry.0.lock().unwrap().take();
    match stored {
        Some((position, size)) => {
            // Size before position, for the same reason `grow` does: the window's height at the
            // moment of the call is what its top-left position is computed against.
            window.set_size(size).map_err(|e| e.to_string())?;
            window.set_position(position).map_err(|e| e.to_string())
        }
        None => {
            let restore = (window.outer_position().map_err(|e| e.to_string())?, window.inner_size().map_err(|e| e.to_string())?);
            geometry.grow(&window, work_area(&window)?, restore)
        }
    }
}

/// The screen area under the window right now, minus the menu bar and the Dock. `None` from
/// `current_monitor` means no monitor claims the window, which a real desktop never does.
fn work_area<R: Runtime>(window: &WebviewWindow<R>) -> Result<PhysicalRect<i32, u32>, String> {
    let monitor = window.current_monitor().map_err(|e| e.to_string())?;
    monitor.map(|monitor| *monitor.work_area()).ok_or_else(|| "No monitor is showing this window".to_string())
}

/// AppKit only grants native full screen (the View menu's "Enter Full Screen", `^⌘F`, and the
/// hidden green-button equivalent) to a window whose collection behavior says it supports one; a
/// titled window gets that for free, but this one has no title bar (`decorations(false)` in
/// `main.rs`), so without this the menu item and shortcut both silently do nothing.
#[cfg(target_os = "macos")]
pub fn allow_native_fullscreen<R: Runtime>(window: &WebviewWindow<R>) -> tauri::Result<()> {
    use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior};
    let ns_window: &NSWindow = unsafe { &*window.ns_window()?.cast() };
    ns_window.setCollectionBehavior(ns_window.collectionBehavior() | NSWindowCollectionBehavior::FullScreenPrimary);
    Ok(())
}
