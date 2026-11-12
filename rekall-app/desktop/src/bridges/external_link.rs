use tauri::{AppHandle, Runtime, Url};
use tauri_plugin_opener::OpenerExt;

/// Opens a web page in the user's browser. Only `http` and `https` are opened: the page is the
/// console's, and nothing it passes may start another program.
#[tauri::command]
pub fn open_external_link<R: Runtime>(app: AppHandle<R>, url: String) -> Result<(), String> {
    let link = web_link_of(&url)?;
    app.opener().open_url(link.as_str(), None::<&str>).map_err(|failure| failure.to_string())
}

fn web_link_of(url: &str) -> Result<Url, String> {
    let link = Url::parse(url.trim()).map_err(|failure| format!("{url} is not a link: {failure}"))?;
    match link.scheme() {
        "http" | "https" => Ok(link),
        scheme => Err(format!("A {scheme} link cannot be opened from the console.")),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/bridges/external_link_tests.rs"]
mod tests;
