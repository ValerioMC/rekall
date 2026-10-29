

/// A component to create or edit. On an edit, a field left out keeps what the component has; on a
/// create, a name left out is the folder's own name.
#[derive(Clone, Debug, Default)]
pub struct ProjectComponentCommand {
    pub name: Option<String>,
    pub folder: Option<String>,
    pub description: Option<String>,
}
