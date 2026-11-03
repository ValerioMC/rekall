use serde::Serialize;

const SHOWN_PATHS: usize = 3;

/// Whether the code a wrapup names has changed since it was written. `Unknown` is every case
/// where git cannot say, so that no false alarm is raised.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum Freshness {
    Fresh,
    PossiblyStale {
        #[serde(rename = "changedPaths")]
        changed_paths: Vec<String>,
        commits: Vec<String>,
    },
    #[default]
    Unknown,
}

impl Freshness {
    /// The line a session and the console read; none unless the wrapup is possibly stale.
    pub fn warning(&self) -> Option<String> {
        let Freshness::PossiblyStale { changed_paths, commits } = self else {
            return None;
        };
        let shown: Vec<&str> = changed_paths.iter().take(SHOWN_PATHS).map(String::as_str).collect();
        let more = if changed_paths.len() > SHOWN_PATHS { ", …" } else { "" };
        let count = commits.len();
        Some(format!(
            "possibly stale: {}{more} changed in {count} commit{} since it was written",
            shown.join(", "),
            if count == 1 { "" } else { "s" }
        ))
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/wrapup/model/freshness_tests.rs"]
mod tests;
