use async_trait::async_trait;
use rekall_service::Services;
use serde_json::Value;

use super::{told, Anchor, AnchoredTask, Arguments, QUALIFY_WITH_PROJECT};
use crate::protocol::{McpTool, ToolError, ToolSchema};
use crate::texts;

/// Lets a planning session put the checklist it proposes on the task, as drafts. It can only ever
/// create a draft: nothing it writes is work until a person promotes it in the console.
pub struct StepProposalTool {
    services: Services,
}

impl StepProposalTool {
    pub fn new(services: Services) -> Self {
        Self { services }
    }
}

#[async_trait]
impl McpTool for StepProposalTool {
    fn name(&self) -> &'static str {
        "rekall_propose_step"
    }

    fn writes(&self) -> bool {
        true
    }

    fn description(&self) -> &'static str {
        texts::PROPOSE_STEP
    }

    fn input_schema(&self) -> Value {
        ToolSchema::object()
            .required_string(
                "anchors",
                "The task to propose the step for, e.g. `project:vega task:report-builder`. Labels, never titles, and \
                 it has to name exactly one task.",
            )
            .required_string("title", "What the step delivers, as a short imperative line.")
            .optional_string(
                "detail",
                "Markdown: what to build, where, what it must satisfy, and how a reviewer can tell it is done.",
            )
            .build()
    }

    async fn execute(&self, arguments: Option<&Value>) -> Result<String, ToolError> {
        let args = Arguments::of(arguments);
        let target = AnchoredTask::from(&Anchor::parse_all(Some(&args.required_string("anchors")?))?)?;
        let title = args.required_string("title")?;
        let detail = args.optional_string("detail");
        let proposed = self
            .services
            .steps
            .propose(target.project_label.as_deref(), &target.task_label, Some(&title), detail.as_deref())
            .await
            .map_err(|e| told(e, QUALIFY_WITH_PROJECT))?;
        let count = proposed.draft_count;
        Ok(texts::PROPOSED_REPORT
            .replacen("%s", &proposed.step.title, 1)
            .replacen("%d", &count.to_string(), 1)
            .replacen("%s", if count == 1 { "" } else { "s" }, 1))
    }
}
