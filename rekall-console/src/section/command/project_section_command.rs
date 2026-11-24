/// A section to create or edit. On an edit, a field left out keeps what the section has. On a
/// create, a context left out puts the section in the project's first one.
#[derive(Clone, Debug, Default)]
pub struct ProjectSectionCommand {
    pub title: Option<String>,
    pub body_markdown: Option<String>,
    pub context_id: Option<String>,
}
