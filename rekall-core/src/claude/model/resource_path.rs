use rekall_kernel::{RekallError, Result};

use crate::claude::ResourceKind;

const MARKDOWN: &str = ".md";
const MEMORY_INDEX: &str = "MEMORY.md";
const SKILL_FILE: &str = "SKILL.md";
const INSTRUCTIONS: &str = "CLAUDE.md";
const PROVIDED_BY_OTHERS: [&str; 2] = ["skills/synced/", "plugins/"];
const PLUGIN_CACHE: &str = "plugins/cache/";
const CLAUDE_FOLDER: &str = ".claude";
const MAX_SEGMENTS: usize = 9;

/// A file of Claude Code's home folder the console may touch, named by its `/`-separated path
/// under it. Parsing is the whole guard: a path outside the known places, or that climbs, hides
/// or carries a separator, is refused before any disk is read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourcePath {
    relative: String,
    kind: ResourceKind,
}

impl ResourcePath {
    pub fn parse(raw: &str) -> Result<Self> {
        let relative = raw.trim();
        let segments: Vec<&str> = relative.split('/').collect();
        let in_plugin_cache = relative.starts_with(PLUGIN_CACHE);
        let plain = |segment: &&str| is_plain_segment(segment) || (in_plugin_cache && *segment == CLAUDE_FOLDER);
        if relative.is_empty() || segments.len() > MAX_SEGMENTS || !segments.iter().all(plain) {
            return Err(refused(raw));
        }
        let kind = classify(&segments).ok_or_else(|| refused(raw))?;
        Ok(Self { relative: relative.to_string(), kind })
    }

    pub fn as_str(&self) -> &str {
        &self.relative
    }

    pub fn kind(&self) -> ResourceKind {
        self.kind
    }

    /// Skills synced from claude.ai or shipped by a plugin are rewritten by their provider, so an
    /// edit would be lost.
    pub fn is_editable(&self) -> bool {
        !PROVIDED_BY_OTHERS.iter().any(|prefix| self.relative.starts_with(prefix))
    }

    pub fn file_name(&self) -> &str {
        self.relative.rsplit('/').next().unwrap_or(&self.relative)
    }

    /// The `MEMORY.md` beside a memory file; none for any other kind.
    pub fn memory_index(&self) -> Option<ResourcePath> {
        if self.kind != ResourceKind::Memory {
            return None;
        }
        let folder = self.relative.rsplit_once('/').map_or("", |(folder, _)| folder);
        Some(Self { relative: format!("{folder}/{MEMORY_INDEX}"), kind: ResourceKind::MemoryIndex })
    }
}

fn refused(raw: &str) -> RekallError {
    RekallError::illegal(format!("\"{raw}\" is not a file Rekall edits in the Claude folder."))
}

/// Not empty, not `.`/`..`, not hidden, and without a character a file name cannot carry.
fn is_plain_segment(segment: &str) -> bool {
    !segment.is_empty()
        && !segment.starts_with('.')
        && !segment.chars().any(|c| c.is_control() || matches!(c, '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'))
}

fn classify(segments: &[&str]) -> Option<ResourceKind> {
    let (file, folders) = segments.split_last()?;
    if !file.ends_with(MARKDOWN) || file.len() == MARKDOWN.len() {
        return None;
    }
    match folders {
        [] if *file == INSTRUCTIONS => Some(ResourceKind::Instructions),
        ["skills"] => Some(ResourceKind::Skill),
        ["skills", .., _] if *file == SKILL_FILE => Some(ResourceKind::Skill),
        ["commands", ..] => Some(ResourceKind::Command),
        ["agents", ..] => Some(ResourceKind::Agent),
        ["plugins", "cache", _, _, _, rest @ ..] if *file == SKILL_FILE && is_plugin_skill_folder(rest) => Some(ResourceKind::Skill),
        ["memory"] | ["projects", _, "memory"] => Some(memory_kind(file)),
        _ => None,
    }
}

/// `skills/<name>` or `.claude/skills/<name>` inside an installed plugin.
fn is_plugin_skill_folder(folders: &[&str]) -> bool {
    matches!(folders, ["skills", _] | [".claude", "skills", _])
}

fn memory_kind(file: &str) -> ResourceKind {
    if file == MEMORY_INDEX { ResourceKind::MemoryIndex } else { ResourceKind::Memory }
}

#[cfg(test)]
#[path = "../../../tests/unit/claude/model/resource_path_tests.rs"]
mod tests;
