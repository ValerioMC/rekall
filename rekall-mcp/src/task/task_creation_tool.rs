use async_trait::async_trait;
use rekall_core::flow::FlowPlan;
use rekall_core::task::TaskKind;
use rekall_core::task::{Created, TaskAuthoringService, TaskBrief};
use serde_json::Value;

use crate::flow::plan_of;
use crate::tool::{told, Anchor, Arguments};
use crate::protocol::{McpTool, ToolError, ToolSchema};
use crate::texts;

/// Lets a session create a task or an epic in a project, with a description and a first checklist or
/// flow, optionally under an epic. The task starts `TODO` and its steps start open, sessions and
/// routes included: it can create, nothing else.
pub struct TaskCreationTool {
    tasks: TaskAuthoringService,
}

impl TaskCreationTool {
    pub fn new(tasks: TaskAuthoringService) -> Self {
        Self { tasks }
    }
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
                "Up to 20 steps, in order: `[{\"title\":\"Expose the report as a download\",\"detail\":\"markdown\",\"new_session\":true}]`. \
                 `detail` and `new_session` (true opens a fresh session) are optional. Give a step `routes` \
                 (`[{\"outcome\":\"app-error\",\"to\":3}]`, `to` a step's one-based number or `end`) and optionally \
                 `max_visits` (1 to 99) to make the task a flow.",
            )
            .build()
    }

    async fn execute(&self, arguments: Option<&Value>) -> Result<String, ToolError> {
        let args = Arguments::of(arguments);
        let project = project_label(&Anchor::parse_all(Some(&args.required_string("anchors")?))?)?;
        let plan = plan_or_empty(args.value("steps"))?;
        let brief = TaskBrief {
            title: Some(args.required_string("title")?),
            label: args.optional_string("label"),
            description: args.optional_string("description"),
            kind: kind_of(args.optional_string("kind").as_deref())?,
            epic: args.optional_string("epic"),
        };
        let created = self.tasks.create(&project, &brief, &plan).await.map_err(|e| told(e, ""))?;
        Ok(report(&created))
    }
}

fn report(created: &Created) -> String {
    if created.kind == TaskKind::Epic {
        return texts::EPIC_CREATED_REPORT.replacen("%s", &created.title, 1).replacen("%s", &created.task_anchor, 2);
    }
    if created.routes > 0 {
        return texts::TASK_CREATED_FLOW_REPORT
            .replacen("%s", &created.title, 1)
            .replacen("%s", &created.task_anchor, 1)
            .replacen("%d", &created.steps.to_string(), 1)
            .replacen("%s", if created.steps == 1 { "" } else { "s" }, 1)
            .replacen("%d", &created.routes.to_string(), 1)
            .replacen("%s", if created.routes == 1 { "" } else { "s" }, 1)
            .replacen("%s", &created.task_anchor, 1);
    }
    texts::TASK_CREATED_REPORT
        .replacen("%s", &created.title, 1)
        .replacen("%s", &created.task_anchor, 1)
        .replacen("%d", &created.steps.to_string(), 1)
        .replacen("%s", if created.steps == 1 { "" } else { "s" }, 1)
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

fn plan_or_empty(argument: Option<&Value>) -> Result<FlowPlan, ToolError> {
    match argument {
        None => Ok(FlowPlan::new(Vec::new())),
        steps => plan_of(steps),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/task/task_creation_tool_tests.rs"]
mod tests;
