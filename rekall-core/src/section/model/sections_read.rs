use crate::section::{ProjectSectionView, SectionContextView};

/// What a session's read came to: the project, its contexts, and either every section in order or
/// the one it asked for by title.
#[derive(Clone, Debug)]
pub struct SectionsRead {
    pub project_anchor: String,
    pub contexts: Vec<SectionContextView>,
    pub sections: Vec<ProjectSectionView>,
    pub asked_for_one: bool,
}
