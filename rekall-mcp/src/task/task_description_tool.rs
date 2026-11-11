use async_trait::async_trait;
use rekall_core::task::{DescriptionReplaced, TaskDescriptionService};
use serde_json::Value;

use crate::protocol::{McpTool, ToolError, ToolSchema};
use crate::texts;
use crate::tool::{told, Anchor, AnchoredTask, Arguments, QUALIFY_WITH_PROJECT};

/// Lets a session replace a task's description with a rewritten text. It changes nothing else, and
/// the text it overwrites is not kept.
pub struct TaskDescriptionTool {
    descriptions: TaskDescriptionService,
}

impl TaskDescriptionTool {
    pub fn new(descriptions: TaskDescriptionService) -> Self {
        Self { descriptions }
    }
}

#[async_trait]
impl McpTool for TaskDescriptionTool {
    fn name(&self) -> &'static str {
        "rekall_description"
    }

    fn writes(&self) -> bool {
        true
    }

    fn description(&self) -> &'static str {
        texts::DESCRIPTION
    }

    fn input_schema(&self) -> Value {
        ToolSchema::object()
            .required_string(
                "anchors",
                "The task whose description is replaced, e.g. `project:vega task:report-builder`. Labels, never titles, \
                 and it has to name exactly one task.",
            )
            .required_string("body", "The complete new description, in markdown. It replaces the whole of the old one.")
            .build()
    }

    async fn execute(&self, arguments: Option<&Value>) -> Result<String, ToolError> {
        let args = Arguments::of(arguments);
        let target = AnchoredTask::from(&Anchor::parse_all(Some(&args.required_string("anchors")?))?)?;
        let body = args.required_string("body")?;
        let replaced = self
            .descriptions
            .replace(target.project_label.as_deref(), &target.task_label, Some(&body))
            .await
            .map_err(|e| told(e, QUALIFY_WITH_PROJECT))?;
        Ok(report(&replaced))
    }
}

fn report(replaced: &DescriptionReplaced) -> String {
    if replaced.changed {
        return format!("Description of `{}` replaced. The earlier text is not kept anywhere.", replaced.task_anchor);
    }
    format!("Description of `{}` already read exactly this: nothing changed.", replaced.task_anchor)
}

#[cfg(test)]
#[path = "../../tests/unit/task/task_description_tool_tests.rs"]
mod tests;
