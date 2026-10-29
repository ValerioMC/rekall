use std::fs;
use std::path::{Path, PathBuf};

use tracing::warn;

use super::plugin_skills::plugin_skill_files;
use super::read_summary;
use super::{ClaudeHome, MemoryGroup, ResourceCatalog, ResourceKind, ResourcePath, ResourceSummary, SkillSource, SyncedSources, WorkingDirectory};

const INSTRUCTIONS: &str = "CLAUDE.md";
const SKILL_FILE: &str = "SKILL.md";
const MAX_DEPTH: usize = 4;

/// Walks the Claude folder and lists what the console edits. A file that cannot be described is
/// logged and left out, so one unreadable memory does not blank the screen.
pub struct ResourceScanner<'a> {
    home: &'a ClaudeHome,
}

impl<'a> ResourceScanner<'a> {
    pub fn new(home: &'a ClaudeHome) -> Self {
        Self { home }
    }

    pub fn catalog(&self) -> ResourceCatalog {
        let root = self.home.root();
        ResourceCatalog {
            root: root.to_string_lossy().into_owned(),
            instructions: self.describe_all([root.join(INSTRUCTIONS)]).pop(),
            skills: self.skills(),
            commands: self.describe_all(markdown_files(&root.join("commands"))),
            agents: self.describe_all(markdown_files(&root.join("agents"))),
            memory_groups: self.memory_groups(),
        }
    }

    /// The person's own skills, those claude.ai synced, and those installed plugins ship; each
    /// carries where it comes from.
    fn skills(&self) -> Vec<ResourceSummary> {
        let root = self.home.root();
        let synced = SyncedSources::read(&root.join("skills").join("synced"));
        let mut skills: Vec<ResourceSummary> = self
            .describe_all(skill_files(&root.join("skills"), 0))
            .into_iter()
            .map(|skill| with_source(skill, &synced))
            .collect();
        for plugin in plugin_skill_files(root) {
            if let Some(skill) = self.describe(&plugin.file) {
                skills.push(ResourceSummary { source: Some(plugin.source), ..skill });
            }
        }
        skills.sort_by_cached_key(|skill| (skill.name.to_lowercase(), skill.path.clone()));
        skills
    }

    fn memory_groups(&self) -> Vec<MemoryGroup> {
        let root = self.home.root();
        let mut groups: Vec<MemoryGroup> = self.memory_group("global", &root.join("memory"), None).into_iter().collect();
        let mut projects = subfolders(&root.join("projects"));
        projects.sort();
        for project in projects {
            let Some(name) = project.file_name().map(|name| name.to_string_lossy().into_owned()) else { continue };
            let directory = WorkingDirectory::recover(&project, &name);
            groups.extend(self.memory_group(&name, &project.join("memory"), Some(directory)));
        }
        groups
    }

    /// A group exists only while its folder holds at least one memory.
    fn memory_group(&self, id: &str, folder: &Path, directory: Option<WorkingDirectory>) -> Option<MemoryGroup> {
        let (index, memories): (Vec<_>, Vec<_>) =
            self.describe_all(markdown_files(folder)).into_iter().partition(|summary| summary.kind == ResourceKind::MemoryIndex);
        if memories.is_empty() {
            return None;
        }
        Some(MemoryGroup {
            id: id.to_string(),
            global: directory.is_none(),
            folder_exists: directory.as_ref().is_some_and(|found| Path::new(&found.path).is_dir()),
            resolved: directory.as_ref().is_none_or(|found| found.resolved),
            working_directory: directory.map(|found| found.path),
            index: index.into_iter().next(),
            memories,
        })
    }

    fn describe_all(&self, files: impl IntoIterator<Item = PathBuf>) -> Vec<ResourceSummary> {
        let mut summaries: Vec<ResourceSummary> = files.into_iter().filter_map(|file| self.describe(&file)).collect();
        summaries.sort_by_cached_key(|summary| summary.name.to_lowercase());
        summaries
    }

    fn describe(&self, file: &Path) -> Option<ResourceSummary> {
        let relative = relative_path(self.home.root(), file)?;
        let path = ResourcePath::parse(&relative).ok()?;
        match read_summary(self.home, &path) {
            Ok(summary) => Some(summary),
            Err(rekall_kernel::RekallError::NotFound(_)) => None,
            Err(failure) => {
                warn!(%failure, path = %relative, "Skipped a Claude file that could not be read");
                None
            }
        }
    }
}

/// A skill under `skills/synced/<bucket>/` is claude.ai's; any other is the person's.
fn with_source(skill: ResourceSummary, synced: &SyncedSources) -> ResourceSummary {
    let segments: Vec<&str> = skill.path.split('/').collect();
    let source = match segments.as_slice() {
        ["skills", "synced", bucket, name, _] => SkillSource::synced(synced.source_of(bucket, name)),
        _ => SkillSource::own(),
    };
    ResourceSummary { source: Some(source), ..skill }
}

fn relative_path(root: &Path, file: &Path) -> Option<String> {
    let relative = file.strip_prefix(root).ok()?;
    Some(relative.components().map(|part| part.as_os_str().to_string_lossy()).collect::<Vec<_>>().join("/"))
}

fn subfolders(folder: &Path) -> Vec<PathBuf> {
    entries_of(folder).into_iter().filter(|entry| entry.is_dir()).collect()
}

fn entries_of(folder: &Path) -> Vec<PathBuf> {
    match fs::read_dir(folder) {
        Ok(entries) => entries.filter_map(|entry| entry.ok().map(|entry| entry.path())).collect(),
        Err(_) => Vec::new(),
    }
}

fn is_markdown(path: &Path) -> bool {
    path.is_file() && path.extension().is_some_and(|extension| extension == "md")
}

fn is_hidden(path: &Path) -> bool {
    path.file_name().is_some_and(|name| name.to_string_lossy().starts_with('.'))
}

/// Every markdown file under `folder`, to `MAX_DEPTH` levels of subfolders.
fn markdown_files(folder: &Path) -> Vec<PathBuf> {
    markdown_files_below(folder, MAX_DEPTH)
}

fn markdown_files_below(folder: &Path, depth_left: usize) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for entry in entries_of(folder).into_iter().filter(|entry| !is_hidden(entry)) {
        if is_markdown(&entry) {
            found.push(entry);
        } else if entry.is_dir() && depth_left > 1 {
            found.extend(markdown_files_below(&entry, depth_left - 1));
        }
    }
    found
}

/// A skill is a markdown file beside the others in `skills/`, or a folder with a `SKILL.md`. A
/// folder without one is a bucket of skills and is searched.
fn skill_files(folder: &Path, depth: usize) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for entry in entries_of(folder).into_iter().filter(|entry| !is_hidden(entry)) {
        let marker = entry.join(SKILL_FILE);
        if entry.is_dir() && marker.is_file() {
            found.push(marker);
        } else if entry.is_dir() && depth < MAX_DEPTH {
            found.extend(skill_files(&entry, depth + 1));
        } else if depth == 0 && is_markdown(&entry) {
            found.push(entry);
        }
    }
    found
}

#[cfg(test)]
#[path = "../../tests/unit/resources/resource_scanner_tests.rs"]
mod tests;
