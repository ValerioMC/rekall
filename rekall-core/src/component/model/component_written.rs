/// What a session's write came to: the component, the project it landed on, how many components
/// that project now has, and whether an existing one was rewritten.
#[derive(Clone, Debug)]
pub struct ComponentWritten {
    pub name: String,
    pub project_anchor: String,
    pub components_on_project: usize,
    pub replaced: bool,
}
