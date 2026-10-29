use rekall_kernel::{Instant, Result};
use serde::Serialize;

use super::claude_home::modified_of;
use super::{ClaudeHome, Frontmatter, ResourceKind, ResourcePath, SkillSource};

const HEAD_BYTES: u64 = 8 * 1024;
const BLURB_CHARS: usize = 160;

/// What the console lists about a file without opening it.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceSummary {
    /// The identity: the file's path under the Claude folder.
    pub path: String,
    pub kind: ResourceKind,
    pub name: String,
    pub description: Option<String>,
    /// A memory's own `type` (`feedback`, `project`...), when its front matter names one.
    pub memory_type: Option<String>,
    pub bytes: u64,
    pub modified_at: Instant,
    pub editable: bool,
    /// Who provides a skill; none for every other kind of file.
    pub source: Option<SkillSource>,
}

impl ResourceSummary {
    pub fn read(home: &ClaudeHome, path: &ResourcePath) -> Result<Self> {
        let metadata = home.metadata(path)?;
        let head = home.head(path, HEAD_BYTES)?;
        let front = Frontmatter::parse(&head);
        let description = front.description.or_else(|| first_prose_line(&head));
        let memory_type = if path.kind() == ResourceKind::Memory { front.kind } else { None };
        Ok(Self {
            path: path.as_str().to_string(),
            kind: path.kind(),
            name: front.name.unwrap_or_else(|| display_name(path)),
            description,
            memory_type,
            bytes: metadata.len(),
            modified_at: modified_of(&metadata),
            editable: path.is_editable(),
            source: None,
        })
    }
}

/// The name a person knows it by: a command is `/folder:name`, a skill its folder.
fn display_name(path: &ResourcePath) -> String {
    let segments: Vec<&str> = path.as_str().split('/').collect();
    let stem = |file: &str| file.strip_suffix(".md").unwrap_or(file).to_string();
    match path.kind() {
        ResourceKind::Instructions | ResourceKind::MemoryIndex => path.file_name().to_string(),
        ResourceKind::Skill if path.file_name() == "SKILL.md" => segments[segments.len() - 2].to_string(),
        ResourceKind::Command | ResourceKind::Agent => segments[1..].iter().map(|s| stem(s)).collect::<Vec<_>>().join(":"),
        ResourceKind::Skill | ResourceKind::Memory => stem(path.file_name()),
    }
}

/// The first line outside the front matter and any code fence that is not a heading.
fn first_prose_line(text: &str) -> Option<String> {
    let body = match text.strip_prefix("---") {
        Some(rest) => rest.split_once("\n---").map_or("", |(_, after)| after),
        None => text,
    };
    let mut in_fence = false;
    for line in body.lines().map(str::trim) {
        if line.starts_with("```") {
            in_fence = !in_fence;
        } else if !in_fence && !line.is_empty() && !line.starts_with('#') && line != "---" {
            return Some(line.chars().take(BLURB_CHARS).collect());
        }
    }
    None
}

#[cfg(test)]
#[path = "../../tests/unit/resources/resource_summary_tests.rs"]
mod tests;
