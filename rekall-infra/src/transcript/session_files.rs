use std::path::PathBuf;

/// One session's transcript and its subagents' transcripts, with their size together.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionFiles {
    pub session_id: String,
    pub main: PathBuf,
    pub subagents: Vec<PathBuf>,
    pub bytes: u64,
}
