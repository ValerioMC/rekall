use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;

#[derive(Deserialize)]
struct Manifest {
    skills: Vec<ManifestEntry>,
}

#[derive(Deserialize)]
struct ManifestEntry {
    name: String,
    source: Option<String>,
}

/// What each `manifest.json` under `skills/synced/` says about where its skills came from.
#[derive(Default)]
pub struct SyncedSources {
    by_bucket: HashMap<String, HashMap<String, String>>,
}

impl SyncedSources {
    pub fn read(synced_folder: &Path) -> Self {
        let Ok(entries) = fs::read_dir(synced_folder) else { return Self::default() };
        let by_bucket = entries
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| Some((entry.file_name().to_string_lossy().into_owned(), manifest_in(&entry.path())?)))
            .collect();
        Self { by_bucket }
    }

    /// The sync's tag for a skill in a bucket, such as `anthropic`.
    pub fn source_of(&self, bucket: &str, skill: &str) -> Option<String> {
        self.by_bucket.get(bucket)?.get(skill).cloned()
    }
}

fn manifest_in(bucket: &Path) -> Option<HashMap<String, String>> {
    let text = fs::read_to_string(bucket.join("manifest.json")).ok()?;
    let manifest: Manifest = serde_json::from_str(&text).ok()?;
    Some(manifest.skills.into_iter().filter_map(|entry| Some((entry.name, entry.source?))).collect())
}
