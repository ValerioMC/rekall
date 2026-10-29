/// What a session's write came to: the section, the project it landed on, how many sections that
/// project now has, and whether an existing one was rewritten.
#[derive(Clone, Debug)]
pub struct SectionWritten {
    pub title: String,
    pub project_anchor: String,
    pub sections_on_project: usize,
    pub replaced: bool,
}
