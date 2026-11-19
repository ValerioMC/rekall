/// Which of a project's texts a write changed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ProjectTextsWritten {
    pub description: bool,
    pub blueprint: bool,
}
