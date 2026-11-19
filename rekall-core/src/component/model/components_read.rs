use crate::component::ProjectComponentView;

/// What a session's read came to: the project, and either every component in order or the one it
/// asked for by name.
#[derive(Clone, Debug)]
pub struct ComponentsRead {
    pub project_anchor: String,
    pub components: Vec<ProjectComponentView>,
    pub asked_for_one: bool,
}
