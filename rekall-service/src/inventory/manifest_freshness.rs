//! Whether the files an inventory was read from changed after it was recorded. An inventory is
//! written once by a session and nothing re-reads it on its own, so a lockfile touched since is the
//! one sign that the list a scan checks no longer matches the project.

use std::collections::BTreeSet;
use std::path::Path;

use chrono::{DateTime, Utc};
use rekall_common::{jstr, Instant};
use rekall_model::project_dependency;

/// The manifests of `rows` whose file in `folder` was modified after `recorded_at`, or is gone; none
/// without a folder, since then nothing can be read.
pub fn changed_since(folder: Option<&str>, rows: &[project_dependency::Model], recorded_at: Instant) -> Vec<String> {
    let Some(folder) = folder.filter(|folder| !jstr::is_blank(folder)) else {
        return Vec::new();
    };
    let root = Path::new(jstr::strip(folder));
    let manifests: BTreeSet<&str> = rows.iter().map(|row| row.manifest.as_str()).collect();
    manifests.into_iter().filter(|manifest| !unchanged_since(&root.join(manifest), recorded_at)).map(str::to_string).collect()
}

fn unchanged_since(file: &Path, recorded_at: Instant) -> bool {
    let modified = std::fs::metadata(file).and_then(|metadata| metadata.modified());
    modified.is_ok_and(|time| !Instant::from_datetime(DateTime::<Utc>::from(time)).is_after(&recorded_at))
}

#[cfg(test)]
#[path = "../../tests/unit/inventory/manifest_freshness_tests.rs"]
mod tests;
