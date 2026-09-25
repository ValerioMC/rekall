use async_trait::async_trait;
use rekall_model::TaskStepState;
use rekall_service::commit::AutoCommitOutcome;
use rekall_service::step::TaskStepView;
use rekall_service::Services;
use serde_json::Value;

use super::{told, Anchor, AnchoredTask, Arguments, QUALIFY_WITH_PROJECT};
use crate::protocol::{McpTool, ToolError, ToolSchema};
use crate::texts;

pub struct StepStateTool {
    services: Services,
}

impl StepStateTool {
    pub fn new(services: Services) -> Self {
        Self { services }
    }

    async fn report(
        &self,
        target: &AnchoredTask,
        moved: &TaskStepView,
        raw_state: &str,
        committed: Option<&AutoCommitOutcome>,
    ) -> Result<String, ToolError> {
        let anchor = target.anchor();
        let all = self.services.steps.find_by_task(moved.task_id).await?;
        let one_based = moved.position + 1;
        let mut out = format!(
            "Step {one_based} \"{}\" on `{anchor}` is now `{}`.\n",
            moved.title,
            moved.state.name().to_lowercase()
        );
        if moved.state == TaskStepState::Claimed {
            if meant_done(raw_state) {
                out.push_str("\nMarked `claimed`, not done: a session cannot tick the last box. ");
            } else {
                out.push_str("\nThe work is finished and waiting for the console to accept it. ");
            }
            out.push_str("Write the wrapup for this task now if you have not, folding in what this step built.\n");
            if let Some(committed) = committed.filter(|c| !c.off()) {
                out.push('\n');
                out.push_str(&committed.describe());
                out.push('\n');
            }
        }
        match all.iter().find(|step| step.state == TaskStepState::Open) {
            Some(next) => out.push_str(&format!(
                "\nNext open step: {} \"{}\". Start it with `/rk {anchor} step:{} start`.",
                next.position + 1,
                next.title,
                next.position + 1
            )),
            None => {
                if !all.is_empty() && all.iter().all(|step| step.state.complete()) {
                    out.push_str("\nEvery step is claimed or done. This task is finished bar the review.");
                }
            }
        }
        Ok(out)
    }
}

fn parse_state(raw: &str) -> Result<TaskStepState, ToolError> {
    let value = rekall_common::jstr::strip(raw).to_lowercase().replace(['-', ' '], "_");
    match value.as_str() {
        "running" | "run" | "start" | "started" | "starting" | "begin" | "in_progress" | "progress" => Ok(TaskStepState::Running),
        "claimed" | "claim" | "done" | "complete" | "completed" | "finish" | "finished" => Ok(TaskStepState::Claimed),
        "open" | "reopen" | "stop" | "abandon" | "todo" | "back" => Ok(TaskStepState::Open),
        _ => Err(ToolError::Failure(format!("`state` is one of `running`, `claimed` or `open`. Got `{raw}`."))),
    }
}

fn meant_done(raw: &str) -> bool {
    let value = rekall_common::jstr::strip(raw).to_lowercase();
    value == "done" || value == "complete" || value == "completed"
}

#[async_trait]
impl McpTool for StepStateTool {
    fn name(&self) -> &'static str {
        "rekall_step"
    }

    fn writes(&self) -> bool {
        true
    }

    fn description(&self) -> &'static str {
        texts::STEP
    }

    fn input_schema(&self) -> Value {
        ToolSchema::object()
            .required_string(
                "anchors",
                "The task the step is on, e.g. `project:vega task:report-builder`. Labels, never titles, and it has to \
                 name exactly one task.",
            )
            .required_string("step", "Which step: its one-based position in the checklist (`\"3\"`), or its exact title.")
            .required_string(
                "state",
                "`running` when you start it, `claimed` when you are finished with it, or `open` to put it back. Never \
                 `done`: the console ticks that.",
            )
            .optional_string(
                "commit_message",
                "Only with `claimed`, and only read on an auto-commit project: the commit message. First line a \
                 Conventional Commits subject (`feat: …`, `fix: …`), under 72 characters. Then a blank line and a body \
                 of two to five sentences on what the change does and why, in plain prose. No list of files.",
            )
            .build()
    }

    async fn execute(&self, arguments: Option<&Value>) -> Result<String, ToolError> {
        let args = Arguments::of(arguments);
        let target = AnchoredTask::from(&Anchor::parse_all(Some(&args.required_string("anchors")?))?)?;
        let step_ref = args.required_string("step")?;
        let raw_state = args.required_string("state")?;
        let commit_message = args.optional_string("commit_message");
        let requested = parse_state(&raw_state)?;

        let moved = self
            .services
            .steps
            .transition(target.project_label.as_deref(), &target.task_label, Some(&step_ref), requested)
            .await
            .map_err(|e| told(e, QUALIFY_WITH_PROJECT))?;

        let committed = if moved.state == TaskStepState::Claimed {
            Some(self.services.auto_commit.after_step_claim(moved.task_id, moved.id, commit_message.as_deref()).await?)
        } else {
            None
        };
        self.report(&target, &moved, &raw_state, committed.as_ref()).await
    }
}
