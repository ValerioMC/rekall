use async_trait::async_trait;
use rekall_core::deviation::DeviationService;
use serde_json::Value;

use crate::protocol::{McpTool, ToolError, ToolSchema};
use crate::texts;
use crate::tool::{told, Anchor, AnchoredTask, Arguments, QUALIFY_WITH_PROJECT};

/// Lets a session record where its work departed from the plan, on the task and for the step it
/// concerns. A second write for the same step rewrites the first; nothing can be deleted from here.
pub struct DeviationTool {
    deviations: DeviationService,
}

impl DeviationTool {
    pub fn new(deviations: DeviationService) -> Self {
        Self { deviations }
    }
}

#[async_trait]
impl McpTool for DeviationTool {
    fn name(&self) -> &'static str {
        "rekall_deviation"
    }

    fn writes(&self) -> bool {
        true
    }

    fn description(&self) -> &'static str {
        texts::DEVIATION
    }

    fn input_schema(&self) -> Value {
        ToolSchema::object()
            .required_string(
                "anchors",
                "The task the deviation belongs to, e.g. `task:RK-12`. Labels, never titles, and \
                 it has to name exactly one task.",
            )
            .required_string("step", "The number of the step the deviation refers to, counted from 1 as the checklist lists them.")
            .required_string("body", "The complete deviation, in markdown: what was planned, what was built, why, the diagram as built.")
            .build()
    }

    async fn execute(&self, arguments: Option<&Value>) -> Result<String, ToolError> {
        let args = Arguments::of(arguments);
        let target = AnchoredTask::from(&Anchor::parse_all(Some(&args.required_string("anchors")?))?)?;
        let step_number = step_number_of(&args.required_string("step")?)?;
        let body = args.required_string("body")?;
        let written = self
            .deviations
            .write(target.project_label.as_deref(), &target.task_code, step_number, Some(&body))
            .await
            .map_err(|e| told(e, QUALIFY_WITH_PROJECT))?;
        let verb = if written.replaced { "rewritten" } else { "recorded" };
        let count = written.deviations_on_task;
        Ok(format!(
            "Deviation on step {} \"{}\" {verb} on `{}`. The task carries {count} deviation{}, and `/rk {}` loads them with it.",
            written.step_number,
            written.step_title,
            written.task_anchor,
            if count == 1 { "" } else { "s" },
            written.task_anchor
        ))
    }
}

fn step_number_of(text: &str) -> Result<i32, ToolError> {
    text.trim()
        .parse::<i32>()
        .map_err(|_| ToolError::Illegal(format!("'step' is the number of a step, like 3, and '{text}' is not one")))
}
