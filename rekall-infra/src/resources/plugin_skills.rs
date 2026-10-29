use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use tracing::warn;

use super::SkillSource;

const INSTALLED: &str = "plugins/installed_plugins.json";
const SKILL_FILE: &str = "SKILL.md";
const SKILL_FOLDERS: [&str; 2] = ["skills", ".claude/skills"];

#[derive(Deserialize)]
struct Installed {
    plugins: HashMap<String, Vec<Installation>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Installation {
    install_path: PathBuf,
    version: Option<String>,
}

/// A skill file an installed plugin ships, with the plugin that provides it.
pub struct PluginSkillFile {
    pub file: PathBuf,
    pub source: SkillSource,
}

/// The skills of every plugin Claude Code has installed, found through its own record of installs
/// so a superseded version left in the cache is not listed.
pub fn plugin_skill_files(root: &Path) -> Vec<PluginSkillFile> {
    let Some(installed) = read_installed(&root.join(INSTALLED)) else { return Vec::new() };
    let mut found = Vec::new();
    for (key, installations) in installed.plugins {
        let (plugin, marketplace) = key.split_once('@').unwrap_or((key.as_str(), ""));
        for installation in installations.iter().filter(|installation| installation.install_path.starts_with(root)) {
            let detail = detail_of(marketplace, installation.version.as_deref());
            for folder in SKILL_FOLDERS {
                for file in skill_files_in(&installation.install_path.join(folder)) {
                    found.push(PluginSkillFile { file, source: SkillSource::plugin(plugin, detail.clone()) });
                }
            }
        }
    }
    found
}

fn read_installed(file: &Path) -> Option<Installed> {
    let text = fs::read_to_string(file).ok()?;
    match serde_json::from_str(&text) {
        Ok(installed) => Some(installed),
        Err(failure) => {
            warn!(%failure, file = %file.display(), "Skipped the plugin list, which could not be read");
            None
        }
    }
}

fn detail_of(marketplace: &str, version: Option<&str>) -> Option<String> {
    let version = version.map(readable_version);
    match (marketplace.is_empty(), version) {
        (true, None) => None,
        (true, Some(version)) => Some(version),
        (false, None) => Some(marketplace.to_string()),
        (false, Some(version)) => Some(format!("{marketplace} · {version}")),
    }
}

/// `v2.0.1` for a release number, the first seven characters for a commit hash.
fn readable_version(version: &str) -> String {
    if version.starts_with(|c: char| c.is_ascii_digit()) && version.contains('.') {
        return format!("v{version}");
    }
    version.chars().take(7).collect()
}

fn skill_files_in(folder: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(folder) else { return Vec::new() };
    entries.filter_map(|entry| entry.ok()).map(|entry| entry.path().join(SKILL_FILE)).filter(|file| file.is_file()).collect()
}

#[cfg(test)]
#[path = "../../tests/unit/resources/plugin_skills_tests.rs"]
mod tests;
