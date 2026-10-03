use tauri::LogicalSize;

/// Share of the screen's width the splash takes, the way launch screens of desktop applications
/// are sized: a clearly smaller card at the centre, never a full-screen cover.
const WIDTH_SHARE: f64 = 0.4;
const MIN_WIDTH: f64 = 640.0;
const MAX_WIDTH: f64 = 960.0;
/// Proportions of `splash/splash_screen.png` (1376 x 768), so the artwork is never cropped.
const ASPECT_RATIO: f64 = 1376.0 / 768.0;

/// The splash window's size, in logical pixels, for a screen whose usable area is `work_area_width`
/// physical pixels wide at `scale_factor`.
pub fn splash_size(work_area_width: u32, scale_factor: f64) -> LogicalSize<f64> {
    let screen_width = f64::from(work_area_width) / scale_factor;
    let width = (screen_width * WIDTH_SHARE).clamp(MIN_WIDTH, MAX_WIDTH).round();
    LogicalSize::new(width, (width / ASPECT_RATIO).round())
}

#[cfg(test)]
#[path = "../../tests/unit/bridges/splash_size_tests.rs"]
mod tests;
