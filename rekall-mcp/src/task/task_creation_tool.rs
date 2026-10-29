use async_trait::async_trait;
use rekall_service::task::TaskKind;
use rekall_service::task::{Created, StepDraft, TaskAuthoringService, TaskBrief};
use serde::Deserialize;
use serde_json::Value;

use crate::tool::{told, Anchor, Arguments};
use crate::protocol::{McpTool, ToolError, ToolSchema};
use crate::texts;

/// Lets a session create a task or an epic in a project, with a description and a first checklist,
/// optionally under an epic. The task starts `TODO` and every step starts as a draft a person has to
/// promote: it can create, nothing else.
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
            .optional_string("kind", "`task` (the default) or `epic`: a task that holds no steps and gathers other tasks.")
            .optional_string("epic", "The label of an epic of the same project to file the new task under: `checkout`.")
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
        let brief = TaskBrief {
            title: Some(args.required_string("title")?),
            label: args.optional_string("label"),
            description: args.optional_string("description"),
            kind: kind_of(args.optional_string("kind").as_deref())?,
            epic: args.optional_string("epic"),
        };
        let created = self.tasks.create(&project, &brief, &drafts).await.map_err(|e| told(e, ""))?;
        Ok(report(&created))
    }
}

fn report(created: &Created) -> String {
    if created.kind == TaskKind::Epic {
        return texts::EPIC_CREATED_REPORT.replacen("%s", &created.title, 1).replacen("%s", &created.task_anchor, 2);
    }
    texts::TASK_CREATED_REPORT
        .replacen("%s", &created.title, 1)
        .replacen("%s", &created.task_anchor, 1)
        .replacen("%d", &created.drafts.to_string(), 1)
        .replacen("%s", if created.drafts == 1 { "" } else { "s" }, 1)
        .replacen("%s", &created.task_anchor, 1)
}

/// `task` or `epic`, in any case; omitted, a task.
fn kind_of(argument: Option<&str>) -> Result<TaskKind, ToolError> {
    match argument.map(|kind| kind.trim().to_ascii_uppercase()) {
        None => Ok(TaskKind::Task),
        Some(kind) => TaskKind::value_of(&kind)
            .ok_or_else(|| ToolError::Illegal(format!("'kind' is `task` or `epic`, not '{}'.", kind.to_lowercase()))),
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
#[path = "../../tests/unit/task/task_creation_tool_tests.rs"]
mod tests;
