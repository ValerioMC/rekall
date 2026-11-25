use async_trait::async_trait;
use rekall_core::step::{TaskStepService, TaskStepView};
use serde_json::Value;

use crate::protocol::{McpTool, ToolError, ToolSchema};
use crate::texts;
use crate::tool::{told, Anchor, AnchoredTask, Arguments, QUALIFY_WITH_PROJECT};

/// Lets a session reword one step still to do, draft or open: its title and detail and nothing
/// else. Running, claimed and done steps are refused.
pub struct StepEditTool {
    steps: TaskStepService,
}

impl StepEditTool {
    pub fn new(steps: TaskStepService) -> Self {
        Self { steps }
    }
}

#[async_trait]
impl McpTool for StepEditTool {
    fn name(&self) -> &'static str {
        "rekall_edit_step"
    }

    fn writes(&self) -> bool {
        true
    }

    fn description(&self) -> &'static str {
        texts::EDIT_STEP
    }

    fn input_schema(&self) -> Value {
        ToolSchema::object()
            .required_string(
                "anchors",
                "The task the step is on, e.g. `task:RK-12`. Labels, never titles, and it has to name exactly one task.",
            )
            .required_string("step", "The step's one-based number as `rekall_read_steps` lists it (`\"3\"`), or its exact title.")
            .optional_string("title", "The new title, as a short imperative line. Omitted, the title stays.")
            .optional_string("detail", "The new detail in markdown, replacing the whole of the old one. Omitted, the detail stays.")
            .build()
    }

    async fn execute(&self, arguments: Option<&Value>) -> Result<String, ToolError> {
        let args = Arguments::of(arguments);
        let target = AnchoredTask::from(&Anchor::parse_all(Some(&args.required_string("anchors")?))?)?;
        let step = args.required_string("step")?;
        let title = args.optional_string("title");
        let detail = args.optional_string("detail");
        let rewritten = self
            .steps
            .rewrite(target.project_label.as_deref(), &target.task_code, Some(&step), title.as_deref(), detail.as_deref())
            .await
            .map_err(|e| told(e, QUALIFY_WITH_PROJECT))?;
        Ok(report(&rewritten))
    }
}

fn report(step: &TaskStepView) -> String {
    texts::STEP_REWRITTEN_REPORT
        .replacen("%d", &(step.position + 1).to_string(), 1)
        .replacen("%s", &step.title, 1)
}

#[cfg(test)]
#[path = "../../tests/unit/step/step_edit_tool_tests.rs"]
mod tests;
