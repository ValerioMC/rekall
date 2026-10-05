//! Where each download in flight is being saved. WKWebView reports a finished download without
//! its path, so the destination chosen when it was requested is what the shell reveals afterwards.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Default)]
pub struct DownloadTracker {
    destinations: Mutex<HashMap<String, PathBuf>>,
}

impl DownloadTracker {
    pub fn started(&self, url: &str, destination: &Path) {
        self.destinations.lock().unwrap().insert(url.to_string(), destination.to_path_buf());
    }

    /// The saved file: the path the webview reported, else the one recorded at the start. The
    /// record is dropped either way, so a repeat of the same URL starts clean.
    pub fn finished(&self, url: &str, reported: Option<PathBuf>) -> Option<PathBuf> {
        let recorded = self.destinations.lock().unwrap().remove(url);
        reported.or(recorded)
    }
}

#[cfg(test)]
#[path = "../tests/unit/download_tracker_tests.rs"]
mod tests;
