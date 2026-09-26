use tauri::{AppHandle, Runtime};
use tauri_plugin_notification::{NotificationExt, PermissionState};

use super::DesktopNotice;

const TITLE_LIMIT: usize = 120;
const BODY_LIMIT: usize = 300;

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
