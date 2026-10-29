

/// A project's components, by id, in the order they are to be shown.
#[derive(Clone, Debug, Default)]
pub struct ProjectComponentOrderCommand {
    pub component_ids: Option<Vec<String>>,
}
