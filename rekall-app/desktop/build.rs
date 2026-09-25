fn main() {
    // The bridges the console calls are app commands; naming them here generates the `allow-*`
    // permissions the capability grants to the page the server serves.
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "pick_folder",
            "open_in_claude_code",
            "notify",
            "close_window",
            "minimize_window",
            "toggle_maximize_window",
        ]),
    ))
    .expect("the Tauri build step");
}
