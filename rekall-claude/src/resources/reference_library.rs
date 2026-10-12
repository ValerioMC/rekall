use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use tracing::warn;

use super::claude_home::MAX_BYTES;
use super::{ClaudeHome, CoveringDocument, CoveringKind, ResourceCatalog, ResourcePath, ResourceSummary, TextFingerprint};

const INSTRUCTIONS: &str = "CLAUDE.md";
const PROJECT_FOLDERS: [&str; 2] = ["", ".claude"];

/// A document Claude loads that a memory may already repeat.
pub struct ReferenceDocument {
    pub document: CoveringDocument,
    pub fingerprint: TextFingerprint,
}

/// Every `CLAUDE.md` and skill a memory could be redundant with: the global instructions and all
/// skills apply everywhere, a project's `CLAUDE.md` (and those of the folders above it) only to the
/// memories written there.
pub struct ReferenceLibrary {
    everywhere: Vec<ReferenceDocument>,
    by_group: HashMap<String, Vec<ReferenceDocument>>,
}

impl ReferenceLibrary {
    pub fn load(home: &ClaudeHome, catalog: &ResourceCatalog) -> Self {
        let mut everywhere = Vec::new();
        let global = catalog.instructions.iter().map(|summary| (CoveringKind::GlobalInstructions, summary));
        let skills = catalog.skills.iter().map(|summary| (CoveringKind::Skill, summary));
        for (kind, summary) in global.chain(skills) {
            if let Some(document) = read_home_document(home, kind, summary) {
                everywhere.push(document);
            }
        }
        let user_home = home.root().parent().map(Path::to_path_buf);
        let by_group = catalog
            .memory_groups
            .iter()
            .filter(|group| group.resolved && group.folder_exists)
            .filter_map(|group| Some((group.id.clone(), group.working_directory.as_ref()?)))
            .map(|(id, directory)| (id, project_documents(Path::new(directory), user_home.as_deref())))
            .collect();
        Self { everywhere, by_group }
    }

    pub fn applying_to<'a>(&'a self, group_id: &str) -> impl Iterator<Item = &'a ReferenceDocument> {
        self.everywhere.iter().chain(self.by_group.get(group_id).into_iter().flatten())
    }

    /// How many distinct files were read.
    pub fn document_count(&self) -> usize {
        let project: HashSet<&str> = self.by_group.values().flatten().map(|document| document.document.location.as_str()).collect();
        self.everywhere.len() + project.len()
    }
}

fn read_home_document(home: &ClaudeHome, kind: CoveringKind, summary: &ResourceSummary) -> Option<ReferenceDocument> {
    let path = ResourcePath::parse(&summary.path).ok()?;
    match home.read(&path) {
        Ok(body) => Some(ReferenceDocument {
            document: CoveringDocument {
                kind,
                label: summary.name.clone(),
                location: home.absolute(&path).to_string_lossy().into_owned(),
                resource_path: Some(summary.path.clone()),
            },
            fingerprint: TextFingerprint::of(&body),
        }),
        Err(failure) => {
            warn!(%failure, path = %summary.path, "Left a Claude file out of the memory analysis");
            None
        }
    }
}

/// The `CLAUDE.md` of a working directory and of each folder above it, up to the user's home:
/// Claude Code loads them all for a session started there.
fn project_documents(directory: &Path, user_home: Option<&Path>) -> Vec<ReferenceDocument> {
    let mut documents = Vec::new();
    for folder in directory.ancestors().take_while(|folder| user_home.is_none_or(|home| *folder != home && folder.starts_with(home))) {
        for sub in PROJECT_FOLDERS {
            let file = folder.join(sub).join(INSTRUCTIONS);
            if let Some(document) = read_project_document(&file, folder) {
                documents.push(document);
            }
        }
    }
    documents
}

fn read_project_document(file: &Path, folder: &Path) -> Option<ReferenceDocument> {
    let size = fs::metadata(file).ok()?.len();
    if size > MAX_BYTES {
        return None;
    }
    match fs::read_to_string(file) {
        Ok(body) => Some(ReferenceDocument {
            document: CoveringDocument {
                kind: CoveringKind::ProjectInstructions,
                label: format!("{}/CLAUDE.md", folder_name(folder)),
                location: file.to_string_lossy().into_owned(),
                resource_path: None,
            },
            fingerprint: TextFingerprint::of(&body),
        }),
        Err(failure) => {
            warn!(%failure, file = %file.display(), "Left a project CLAUDE.md out of the memory analysis");
            None
        }
    }
}

fn folder_name(folder: &Path) -> String {
    folder.file_name().map_or_else(|| PathBuf::from(folder).to_string_lossy().into_owned(), |name| name.to_string_lossy().into_owned())
}
