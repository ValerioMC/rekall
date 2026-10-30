use async_trait::async_trait;
use rekall_core::step::TaskStepService;
use serde_json::Value;

use crate::protocol::{McpTool, ToolError, ToolSchema};
use crate::texts;
use crate::tool::{told, Anchor, AnchoredTask, Arguments, QUALIFY_WITH_PROJECT};

/// Lets a session delete one step it has not started, draft or open. Running, claimed and done
/// steps carry a run or a review and are refused: only the console discards those.
pub struct StepDeletionTool {
    steps: TaskStepService,
}

impl StepDeletionTool {
    pub fn new(steps: TaskStepService) -> Self {
        Self { steps }
    }
}

#[async_trait]
impl McpTool for StepDeletionTool {
    fn name(&self) -> &'static str {
        "rekall_delete_step"
    }

    fn writes(&self) -> bool {
        true
    }

    fn description(&self) -> &'static str {
        texts::DELETE_STEP
    }

    fn input_schema(&self) -> Value {
        ToolSchema::object()
            .required_string(
                "anchors",
                "The task the step is on, e.g. `project:vega task:report-builder`. Labels, never titles, and it has to name exactly one task.",
            )
            .required_string("step", "The step's one-based number as `rekall_context` lists it (`\"3\"`), or its exact title.")
            .build()
    }

    async fn execute(&self, arguments: Option<&Value>) -> Result<String, ToolError> {
        let args = Arguments::of(arguments);
        let target = AnchoredTask::from(&Anchor::parse_all(Some(&args.required_string("anchors")?))?)?;
        let step = args.required_string("step")?;
        let removed = self
            .steps
            .remove(target.project_label.as_deref(), &target.task_label, Some(&step))
            .await
            .map_err(|e| told(e, QUALIFY_WITH_PROJECT))?;
        Ok(texts::STEP_DELETED_REPORT
            .replacen("%s", &removed.title, 1)
            .replacen("%d", &removed.remaining.to_string(), 1)
            .replacen("%s", if removed.remaining == 1 { "" } else { "s" }, 1))
    }
}
