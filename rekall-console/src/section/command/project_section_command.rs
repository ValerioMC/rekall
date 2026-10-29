

/// A section to create or edit. On an edit, a field left out keeps what the section has.
#[derive(Clone, Debug, Default)]
pub struct ProjectSectionCommand {
    pub title: Option<String>,
    pub body_markdown: Option<String>,
}
