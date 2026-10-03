use std::time::Duration;

use tauri::{Runtime, WebviewWindow};

const FADE_STEPS: u32 = 20;
const FADE_DURATION: Duration = Duration::from_millis(350);

/// Shows the hidden `window` by raising its opacity from nothing to full, instead of cutting it in.
/// Only macOS exposes a window opacity; elsewhere the window is simply shown.
pub async fn show_fading_in<R: Runtime>(window: &WebviewWindow<R>) -> tauri::Result<()> {
    #[cfg(target_os = "macos")]
    {
        set_opacity(window, 0.0)?;
        window.show()?;
        fade(window, 0.0, 1.0).await
    }
    #[cfg(not(target_os = "macos"))]
    {
        window.show()
    }
}

/// Dims the shown `window` to fully transparent, still open, so it can be resized unseen.
pub async fn fade_out<R: Runtime>(window: &WebviewWindow<R>) -> tauri::Result<()> {
    #[cfg(target_os = "macos")]
    {
        fade(window, 1.0, 0.0).await
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = window;
        Ok(())
    }
}

/// Brings a window dimmed by `fade_out` back to full opacity.
pub async fn fade_in<R: Runtime>(window: &WebviewWindow<R>) -> tauri::Result<()> {
    #[cfg(target_os = "macos")]
    {
        fade(window, 0.0, 1.0).await
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = window;
        Ok(())
    }
}

#[cfg(target_os = "macos")]
async fn fade<R: Runtime>(window: &WebviewWindow<R>, from: f64, to: f64) -> tauri::Result<()> {
    for step in 1..=FADE_STEPS {
        tokio::time::sleep(FADE_DURATION / FADE_STEPS).await;
        set_opacity(window, from + (to - from) * f64::from(step) / f64::from(FADE_STEPS))?;
    }
    Ok(())
}

/// AppKit touches a window from the main thread only, so the change is handed to it.
#[cfg(target_os = "macos")]
fn set_opacity<R: Runtime>(window: &WebviewWindow<R>, opacity: f64) -> tauri::Result<()> {
    use objc2_app_kit::NSWindow;
    let target = window.clone();
    window.run_on_main_thread(move || {
        if let Ok(pointer) = target.ns_window() {
            let ns_window: &NSWindow = unsafe { &*pointer.cast() };
            ns_window.setAlphaValue(opacity);
        }
    })
}
