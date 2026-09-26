use std::sync::Mutex;

use tauri::{PhysicalPosition, PhysicalRect, PhysicalSize, Runtime, WebviewWindow};

/// Where the window sat before it was last grown to fill its monitor's work area: `Some` is what
/// the next click restores, `None` means it is already at that restored size. Tracked here rather
/// than asked of the window with `is_maximized`/`maximize`/`unmaximize`, because those delegate to
/// AppKit's native zoom, which on macOS only recognises a titled window as "zoomed" — for a
/// borderless one (`decorations(false)`, see `main.rs`) it falls back to comparing the window's
/// frame to the screen's visible area, so the first click after launch only established that the
/// window was not really zoomed and it took a second click to grow it. Growing and restoring the
/// window directly, from a size this struct remembers itself, sidesteps that native path entirely.
#[derive(Default)]
pub struct WindowGeometry(pub(super) Mutex<Option<(PhysicalPosition<i32>, PhysicalSize<u32>)>>);

impl WindowGeometry {
    pub(super) fn grow<R: Runtime>(
        &self,
        window: &WebviewWindow<R>,
        area: PhysicalRect<i32, u32>,
        restore: (PhysicalPosition<i32>, PhysicalSize<u32>),
    ) -> Result<(), String> {
        *self.0.lock().unwrap() = Some(restore);
        // Size first: macOS positions a window from its bottom-left corner, so computing where
        // the top-left ends up (`set_position`) has to happen after the height it is based on is
        // already correct — setting the position first placed the top edge using the window's
        // still-old height, then growing it afterwards pushed that edge further up the screen.
        window.set_size(area.size).map_err(|e| e.to_string())?;
        window.set_position(area.position).map_err(|e| e.to_string())
    }
}
