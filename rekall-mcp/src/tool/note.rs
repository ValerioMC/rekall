use async_trait::async_trait;
use rekall_service::Services;
use serde_json::Value;

use super::{told, Anchor, AnchoredTask, Arguments, QUALIFY_WITH_PROJECT};
use crate::protocol::{McpTool, ToolError, ToolSchema};
use crate::texts;

/// Lets a session keep what it produced as a note on the task it is working, rather than folding
/// it into a wrapup that is not the place for it. It only ever adds a note: nothing already in
/// Rekall can be edited, detached or deleted through it.
pub struct NoteTool {
    services: Services,
}

impl NoteTool {
    pub fn new(services: Services) -> Self {
        Self { services }
    }
}

#[async_trait]
impl McpTool for NoteTool {
    fn name(&self) -> &'static str {
        "rekall_note"
    }

    fn writes(&self) -> bool {
        true
    }

    fn description(&self) -> &'static str {
        texts::NOTE
    }

    fn input_schema(&self) -> Value {
        ToolSchema::object()
            .required_string(
                "anchors",
                "The task to attach the note to, e.g. `project:vega task:report-builder`. Labels, never titles, and \
                 it has to name exactly one task.",
            )
            .required_string("title", "What the note is, short, like a file name.")
            .required_string("body", "The complete note, in markdown.")
            .build()
    }

    async fn execute(&self, arguments: Option<&Value>) -> Result<String, ToolError> {
        let args = Arguments::of(arguments);
        let target = AnchoredTask::from(&Anchor::parse_all(Some(&args.required_string("anchors")?))?)?;
        let title = args.required_string("title")?;
        let body = args.required_string("body")?;
        let written = self
            .services
            .notes
            .write(target.project_label.as_deref(), &target.task_label, Some(&title), Some(&body))
            .await
            .map_err(|e| told(e, QUALIFY_WITH_PROJECT))?;
        let count = written.notes_on_task;
        Ok(format!(
            "Note \"{}\" written on `{}` as `{}`. The task carries {count} note{}, and `/rk {}` loads this one with it \
             from now on.",
            written.title,
            written.task_anchor,
            written.note_anchor,
            if count == 1 { "" } else { "s" },
            written.task_anchor
        ))
    }
}
