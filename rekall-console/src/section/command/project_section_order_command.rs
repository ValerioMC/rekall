

/// A project's sections, by id, in the order they are to be shown.
#[derive(Clone, Debug, Default)]
pub struct ProjectSectionOrderCommand {
    pub section_ids: Option<Vec<String>>,
}
