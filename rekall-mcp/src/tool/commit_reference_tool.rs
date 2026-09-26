use async_trait::async_trait;
use rekall_service::Services;
use serde_json::Value;

use super::{told, Anchor, AnchoredTask, Arguments, QUALIFY_WITH_PROJECT};
use crate::protocol::{McpTool, ToolError, ToolSchema};
use crate::texts;

pub struct CommitReferenceTool {
    services: Services,
}

impl CommitReferenceTool {
    pub fn new(services: Services) -> Self {
        Self { services }
    }
}

#[async_trait]
impl McpTool for CommitReferenceTool {
    fn name(&self) -> &'static str {
        "rekall_record_commit"
    }

    fn writes(&self) -> bool {
        true
    }

    fn description(&self) -> &'static str {
        texts::RECORD_COMMIT
    }

    fn input_schema(&self) -> Value {
        ToolSchema::object()
            .required_string(
                "anchors",
                "The task to log this commit against, e.g. `project:vega task:report-builder`. Labels, never titles. \
                 It has to name exactly one task.",
            )
            .optional_string(
                "step",
                "Which step, if any: its one-based position in the checklist (`\"3\"`), or its exact title. Omitted \
                 logs the commit against the task itself.",
            )
            .optional_string(
                "commit",
                "The hash of the commit to log, abbreviated or full, when it is not the tip of the repo. Omitted logs \
                 the tip.",
            )
            .build()
    }

    async fn execute(&self, arguments: Option<&Value>) -> Result<String, ToolError> {
        let args = Arguments::of(arguments);
        let target = AnchoredTask::from(&Anchor::parse_all(Some(&args.required_string("anchors")?))?)?;
        let step = args.optional_string("step");
        let commit = args.optional_string("commit");
        let logged = self
            .services
            .commit_references
            .record_commit_by_anchor(target.project_label.as_deref(), &target.task_label, step.as_deref(), commit.as_deref())
            .await
            .map_err(|e| told(e, QUALIFY_WITH_PROJECT))?;
        let where_ = match &logged.step_title {
            None => "the task".to_string(),
            Some(title) => format!("\"{title}\""),
        };
        let short = &logged.commit_hash[..logged.commit_hash.len().min(7)];
        Ok(format!("Logged `{short}` — {} — against {where_}.", logged.comment))
    }
}
