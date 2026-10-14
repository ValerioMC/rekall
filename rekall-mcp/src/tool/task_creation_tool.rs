use async_trait::async_trait;
use rekall_service::task::{StepDraft, TaskAuthoringService};
use serde::Deserialize;
use serde_json::Value;

use super::{told, Anchor, Arguments};
use crate::protocol::{McpTool, ToolError, ToolSchema};
use crate::texts;

/// Lets a session create a task in a project, with a description and a first checklist. The task
/// starts `TODO` and every step starts as a draft a person has to promote: it can create, nothing else.
pub struct TaskCreationTool {
    tasks: TaskAuthoringService,
}

impl TaskCreationTool {
    pub fn new(tasks: TaskAuthoringService) -> Self {
        Self { tasks }
    }
}

#[derive(Deserialize)]
struct StepArgument {
    title: String,
    detail: Option<String>,
}

#[async_trait]
impl McpTool for TaskCreationTool {
    fn name(&self) -> &'static str {
        "rekall_create_task"
    }

    fn writes(&self) -> bool {
        true
    }

    fn description(&self) -> &'static str {
        texts::CREATE_TASK
    }

    fn input_schema(&self) -> Value {
        ToolSchema::object()
            .required_string("anchors", "The project the task goes in, `project:vega`. A label, never a title.")
            .required_string("title", "What the task is, short, under 200 characters.")
            .optional_string(
                "label",
                "The task's label, what `task:` resolves. Omitted, it is the title as a label. It must be free on the project.",
            )
            .optional_string(
                "description",
                "Markdown: what the work is for, what it has to satisfy, what is out of scope.",
            )
            .optional_object_array(
                "steps",
                "Up to 20 drafts, in order: `[{\"title\":\"Expose the report as a download\",\"detail\":\"markdown\"}]`. \
                 `detail` is optional.",
            )
            .build()
    }

    async fn execute(&self, arguments: Option<&Value>) -> Result<String, ToolError> {
        let args = Arguments::of(arguments);
        let project = project_label(&Anchor::parse_all(Some(&args.required_string("anchors")?))?)?;
        let drafts = drafts_of(args.value("steps"))?;
        let created = self
            .tasks
            .create(
                &project,
                Some(&args.required_string("title")?),
                args.optional_string("label").as_deref(),
                args.optional_string("description").as_deref(),
                &drafts,
            )
            .await
            .map_err(|e| told(e, ""))?;
        Ok(texts::TASK_CREATED_REPORT
            .replacen("%s", &created.title, 1)
            .replacen("%s", &created.task_anchor, 1)
            .replacen("%d", &created.drafts.to_string(), 1)
            .replacen("%s", if created.drafts == 1 { "" } else { "s" }, 1)
            .replacen("%s", &created.task_anchor, 1))
    }
}

fn project_label(anchors: &[Anchor]) -> Result<String, ToolError> {
    match anchors {
        [project] if project.is("project") => Ok(project.value.clone()),
        _ => Err(ToolError::Failure("A task is created in one project: pass `project:<label>`.".into())),
    }
}

/// The steps as an array, or as a string holding one: some clients serialise a nested argument.
fn drafts_of(argument: Option<&Value>) -> Result<Vec<StepDraft>, ToolError> {
    let steps: Vec<StepArgument> = match argument {
        None => return Ok(Vec::new()),
        Some(Value::String(text)) => serde_json::from_str(text),
        Some(array) => serde_json::from_value(array.clone()),
    }
    .map_err(|error| ToolError::Illegal(format!("'steps' is not a list of {{title, detail}} objects: {error}")))?;
    Ok(steps.into_iter().map(|step| StepDraft { title: step.title, detail: step.detail }).collect())
}

#[cfg(test)]
#[path = "../../tests/unit/tool/task_creation_tool_tests.rs"]
mod tests;
