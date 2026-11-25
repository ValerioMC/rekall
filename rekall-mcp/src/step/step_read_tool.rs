use async_trait::async_trait;
use rekall_core::step::{StepLinkView, TaskStepView, TaskStepService, UnstartedStepsView};
use serde_json::Value;

use crate::protocol::{McpTool, ToolError, ToolSchema};
use crate::texts;
use crate::tool::{told, Anchor, AnchoredTask, Arguments, QUALIFY_WITH_PROJECT};

/// Lets a session read the draft and open steps of a task as stored, so that it can reword one
/// without losing what a context leaves out or expands; it writes nothing.
pub struct StepReadTool {
    steps: TaskStepService,
}

impl StepReadTool {
    pub fn new(steps: TaskStepService) -> Self {
        Self { steps }
    }
}

#[async_trait]
impl McpTool for StepReadTool {
    fn name(&self) -> &'static str {
        "rekall_read_steps"
    }

    fn description(&self) -> &'static str {
        texts::READ_STEPS
    }

    fn input_schema(&self) -> Value {
        ToolSchema::object()
            .required_string(
                "anchors",
                "The task whose steps to read, e.g. `task:RK-12`. Labels, never titles, and it has to name exactly one task.",
            )
            .build()
    }

    async fn execute(&self, arguments: Option<&Value>) -> Result<String, ToolError> {
        let args = Arguments::of(arguments);
        let target = AnchoredTask::from(&Anchor::parse_all(Some(&args.required_string("anchors")?))?)?;
        let read = self
            .steps
            .read_unstarted(target.project_label.as_deref(), &target.task_code)
            .await
            .map_err(|e| told(e, QUALIFY_WITH_PROJECT))?;
        Ok(report(&read))
    }
}

fn report(read: &UnstartedStepsView) -> String {
    if read.steps.is_empty() {
        return format!("`{}` has no draft or open step.", read.task_anchor);
    }
    let steps: Vec<String> = read.steps.iter().map(|step| render(step, read)).collect();
    format!("Draft and open steps of `{}`, as stored:\n\n{}", read.task_anchor, steps.join("\n\n"))
}

fn render(step: &TaskStepView, read: &UnstartedStepsView) -> String {
    let heading = format!("## Step {} ({}): {}", step.position + 1, step.state.name().to_lowercase(), step.title);
    let routes: Vec<String> = step.links.iter().filter_map(|link| render_route(link, read)).collect();
    let routes = if routes.is_empty() { String::new() } else { format!("\nRoutes: {}", routes.join("; ")) };
    let detail = match &step.body_markdown {
        Some(detail) => format!("<detail>\n{detail}\n</detail>"),
        None => "No detail.".to_string(),
    };
    format!("{heading}{routes}\n{detail}")
}

/// A route to a step that is gone is left out.
fn render_route(link: &StepLinkView, read: &UnstartedStepsView) -> Option<String> {
    let target = match link.to_step_id {
        Some(to_step_id) => format!("step {}", read.number_of(to_step_id)?),
        None => "end of the flow".to_string(),
    };
    Some(format!("`{}` -> {target}", link.outcome))
}

#[cfg(test)]
#[path = "../../tests/unit/step/step_read_tool_tests.rs"]
mod tests;
