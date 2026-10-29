use crate::section::ProjectSectionView;

/// What a session's read came to: the project, and either every section in order or the one it
/// asked for by title.
#[derive(Clone, Debug)]
pub struct SectionsRead {
    pub project_anchor: String,
    pub sections: Vec<ProjectSectionView>,
    pub asked_for_one: bool,
}
