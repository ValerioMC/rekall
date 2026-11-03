/// `folder` is the working directory Claude Code reports for the session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuardContextCommand {
    pub folder: String,
}
