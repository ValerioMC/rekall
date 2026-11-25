use serde::Deserialize;

/// The slash command a local-command line answers, as `claude` names it: `context`, without the slash.
#[derive(Debug, Deserialize)]
pub(in crate::chat) struct WireLocalCommand {
    pub(in crate::chat) command: String,
    #[serde(default)]
    pub(in crate::chat) args: String,
}
