use rekall_kernel::Id;
use rekall_console::note::DocumentCommand;
use rekall_store::DocumentContextMode;
use serde::Deserialize;

use crate::shared::error::ApiResult;
use crate::shared::extract::present;

/// `contextMode` left out keeps what the note had, or `FULL` for a new one.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DocumentRequest {
    pub task_id: Option<Id>,
    pub title: Option<String>,
    pub kind: Option<String>,
    pub body_markdown: Option<String>,
    pub context_mode: Option<DocumentContextMode>,
}

impl DocumentRequest {
    pub fn into_command(self) -> ApiResult<DocumentCommand> {
        Ok(DocumentCommand {
            task_id: present("taskId", self.task_id)?,
            title: self.title,
            kind: self.kind,
            body_markdown: self.body_markdown,
            context_mode: self.context_mode,
        })
    }
}
