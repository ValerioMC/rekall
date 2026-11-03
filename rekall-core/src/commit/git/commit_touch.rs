/// A commit and the files it changed among the ones asked about.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommitTouch {
    pub hash: String,
    pub files: Vec<String>,
}
