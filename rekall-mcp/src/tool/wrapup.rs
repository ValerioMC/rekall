use async_trait::async_trait;
use rekall_model::WrapupAuthor;
use rekall_service::Services;
use serde_json::Value;

use super::{told, Anchor, AnchoredTask, Arguments, QUALIFY_WITH_PROJECT};
use crate::protocol::{McpTool, ToolError, ToolSchema};
use crate::texts;

pub struct WrapupTool {
    services: Services,
}

impl WrapupTool {
    pub fn new(services: Services) -> Self {
        Self { services }
    }
}

#[async_trait]
impl McpTool for WrapupTool {
    fn name(&self) -> &'static str {
        "rekall_wrapup"
    }

    fn writes(&self) -> bool {
        true
    }

    fn description(&self) -> &'static str {
        texts::WRAPUP
    }

    fn input_schema(&self) -> Value {
        ToolSchema::object()
            .required_string(
                "anchors",
                "The task to write to, e.g. `project:vega task:report-builder`. Labels, never titles. It has to name \
                 exactly one task; a `company:` anchor cannot.",
            )
            .required_string(
                "body",
                "The complete wrapup, in markdown. Replaces what was there. Describes the implementation as it stands, \
                 not what changed in this session.",
            )
            .optional_string(
                "commit_message",
                "Only read on a task with no checklist whose project auto-commits: the commit message. First line a \
                 Conventional Commits subject (`feat: …`, `fix: …`), under 72 characters. Then a blank line and a body \
                 of two to five sentences on what the change does and why, in plain prose. No list of files.",
            )
            .build()
    }

    async fn execute(&self, arguments: Option<&Value>) -> Result<String, ToolError> {
        let args = Arguments::of(arguments);
        let target = AnchoredTask::from(&Anchor::parse_all(Some(&args.required_string("anchors")?))?)?;
        let body = args.required_string("body")?;
        let commit_message = args.optional_string("commit_message");

        // The transport stamps the author: anything arriving over MCP was written by Claude.
        let written = self
            .services
            .wrapups
            .write_by_anchor(target.project_label.as_deref(), &target.task_label, Some(&body), WrapupAuthor::Claude)
            .await
            .map_err(|e| told(e, QUALIFY_WITH_PROJECT))?;

        let mut out = format!(
            "{}`{}`.\n",
            if written.created { "Wrapup written for " } else { "Wrapup replaced for " },
            written.wrapup.anchor
        );
        if written.replaced == Some(WrapupAuthor::Hand) {
            out.push_str(
                "\nThe version you replaced had been edited by hand in the console. It is kept in the wrapup's history \
                 there, but if that edit said something this one does not, only a person restoring it brings it back. \
                 Tell them.\n",
            );
        }
        out.push_str(&format!("\nIt is what `/rk {}` will load from now on.", written.wrapup.anchor));

        let committed = self.services.auto_commit.after_wrapup(written.wrapup.task_id, commit_message.as_deref()).await?;
        if !committed.off() {
            out.push_str("\n\n");
            out.push_str(&committed.describe());
        }
        Ok(out)
    }
}
