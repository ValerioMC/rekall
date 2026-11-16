//! What a repository says it does: the description in its manifest (`Cargo.toml`, `package.json`,
//! `pyproject.toml`), or else the first paragraph of prose in its README. Empty when it says nothing.

use std::io;
use std::path::Path;

use rekall_kernel::jstr;
use serde::de::IgnoredAny;
use serde::Deserialize;
use tracing::warn;

/// A component's description travels in every context that loads the project, so it stays short.
const MAX_CHARACTERS: usize = 300;
const README_NAMES: [&str; 6] = ["README.md", "Readme.md", "readme.md", "README.markdown", "README.txt", "README"];

#[derive(Deserialize)]
struct Described {
    description: Option<ManifestDescription>,
}

/// A description a manifest inherits from its workspace is not text, and is read as none.
#[derive(Deserialize)]
#[serde(untagged)]
enum ManifestDescription {
    Text(String),
    Inherited(IgnoredAny),
}

#[derive(Deserialize)]
struct CargoManifest {
    package: Option<Described>,
}

#[derive(Deserialize)]
struct PyprojectManifest {
    project: Option<Described>,
}

pub(crate) fn describe(folder: &Path) -> String {
    let found = cargo_description(folder).or_else(|| package_json_description(folder)).or_else(|| pyproject_description(folder)).or_else(|| readme_description(folder));
    found.map(|text| shortened(&text)).unwrap_or_default()
}

fn cargo_description(folder: &Path) -> Option<String> {
    let manifest: CargoManifest = parsed(&folder.join("Cargo.toml"), |text| toml::from_str(text).map_err(|failure| failure.to_string()))?;
    text_of(manifest.package?)
}

fn pyproject_description(folder: &Path) -> Option<String> {
    let manifest: PyprojectManifest = parsed(&folder.join("pyproject.toml"), |text| toml::from_str(text).map_err(|failure| failure.to_string()))?;
    text_of(manifest.project?)
}

fn package_json_description(folder: &Path) -> Option<String> {
    let manifest: Described = parsed(&folder.join("package.json"), |text| serde_json::from_str(text).map_err(|failure| failure.to_string()))?;
    text_of(manifest)
}

fn text_of(described: Described) -> Option<String> {
    match described.description? {
        ManifestDescription::Text(text) if !jstr::is_blank(&text) => Some(text),
        ManifestDescription::Text(_) | ManifestDescription::Inherited(_) => None,
    }
}

fn readme_description(folder: &Path) -> Option<String> {
    let text = README_NAMES.iter().find_map(|name| read_text(&folder.join(name)))?;
    paragraphs(&text).into_iter().find(|paragraph| is_prose(paragraph)).map(|paragraph| paragraph.join(" "))
}

fn parsed<T>(path: &Path, parse: impl FnOnce(&str) -> Result<T, String>) -> Option<T> {
    let text = read_text(path)?;
    match parse(&text) {
        Ok(manifest) => Some(manifest),
        Err(failure) => {
            warn!(%failure, file = %path.display(), "A manifest could not be read while describing a component");
            None
        }
    }
}

fn read_text(path: &Path) -> Option<String> {
    match std::fs::read_to_string(path) {
        Ok(text) => Some(text),
        Err(failure) if failure.kind() == io::ErrorKind::NotFound => None,
        Err(failure) => {
            warn!(%failure, file = %path.display(), "A file could not be read while describing a component");
            None
        }
    }
}

/// The runs of lines between blank lines, with fenced code left out.
fn paragraphs(text: &str) -> Vec<Vec<&str>> {
    let mut paragraphs: Vec<Vec<&str>> = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    let mut in_fence = false;
    for line in text.lines().map(jstr::strip) {
        if line.starts_with("```") || line.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if line.is_empty() {
            if !current.is_empty() {
                paragraphs.push(std::mem::take(&mut current));
            }
        } else {
            current.push(line);
        }
    }
    if !current.is_empty() {
        paragraphs.push(current);
    }
    paragraphs
}

/// Prose is not a heading (with or without an underline), an image or badge, markup, a table, a
/// quote, a list or a rule.
fn is_prose(paragraph: &[&str]) -> bool {
    let first = paragraph[0];
    let marked = ["#", "![", "[!", "<", "|", ">", "- ", "* ", "+ ", "---", "***"].iter().any(|mark| first.starts_with(mark));
    let underlined = paragraph.iter().skip(1).any(|line| line.chars().all(|c| c == '=') || line.chars().all(|c| c == '-'));
    !marked && !underlined
}

fn shortened(text: &str) -> String {
    let text = jstr::collapse_whitespace(jstr::strip(text));
    if text.chars().count() <= MAX_CHARACTERS {
        return text;
    }
    let cut: String = text.chars().take(MAX_CHARACTERS).collect();
    let words = cut.rsplit_once(' ').map_or(cut.as_str(), |(head, _)| head);
    format!("{}…", words.trim_end_matches([',', ';', ':', '.', ' ']))
}

#[cfg(test)]
#[path = "../../../tests/unit/component/support/component_description_tests.rs"]
mod tests;
