use tauri::{Runtime, WebviewWindow};
use tauri_plugin_dialog::DialogExt;

use super::expand_tilde;

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
