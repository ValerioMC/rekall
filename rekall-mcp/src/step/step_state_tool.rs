use async_trait::async_trait;
use rekall_model::TaskStepState;
use rekall_service::commit::{AutoCommitOutcome, AutoCommitService};
use rekall_service::flow::{FlowMove, FlowRoute, FlowService};
use rekall_service::step::TaskStepService;
use serde_json::Value;

use crate::tool::{told, Anchor, AnchoredTask, Arguments, QUALIFY_WITH_PROJECT};
use crate::protocol::{McpTool, ToolError, ToolSchema};
use crate::texts;

pub struct StepStateTool {
    auto_commit: AutoCommitService,
    steps: TaskStepService,
    flow: FlowService,
}

impl StepStateTool {
    pub fn new(auto_commit: AutoCommitService, steps: TaskStepService, flow: FlowService) -> Self {
        Self { auto_commit, steps, flow }
    }

    async fn report(
        &self,
        target: &AnchoredTask,
        flow_move: &FlowMove,
        raw_state: &str,
        committed: Option<&AutoCommitOutcome>,
    ) -> Result<String, ToolError> {
        let moved = &flow_move.step;
        let anchor = target.anchor();
        let all = self.steps.find_by_task(moved.task_id).await?;
        let one_based = moved.position + 1;
        let mut out = format!(
            "Step {one_based} \"{}\" on `{anchor}` is now `{}`.\n",
            moved.title,
            moved.state.name().to_lowercase()
        );
        if moved.state == TaskStepState::Claimed {
            out.push_str(&claim_note(raw_state, flow_move.route.as_ref()));
            if let Some(committed) = committed.filter(|c| !c.off()) {
                out.push('\n');
                out.push_str(&committed.describe());
                out.push('\n');
            }
        }
        if let Some(route) = &flow_move.route {
            out.push_str(&describe_route(&anchor, route));
            return Ok(out);
        }
        // On a flow the next step is the one a claim routes to, never the next in list order.
        if all.iter().any(|step| !step.links.is_empty()) {
            return Ok(out);
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

/// What a claim means for the session. Mid-flow nobody reviews before the next step starts, so the
/// answer must not read as a hand-off.
fn claim_note(raw_state: &str, route: Option<&FlowRoute>) -> String {
    match route {
        Some(FlowRoute::Followed { .. }) => {
            "\nClaimed. The flow does not wait for the review: carry on with the next step, and write the wrapup once the flow is finished.\n"
                .to_string()
        }
        Some(FlowRoute::Ended { .. }) => "\nClaimed. Write the wrapup for this task now, describing the whole flow.\n".to_string(),
        None => {
            let lead = if meant_done(raw_state) {
                "Marked `claimed`, not done: a session cannot tick the last box. "
            } else {
                "The work is finished and waiting for the console to accept it. "
            };
            format!("\n{lead}Write the wrapup for this task now if you have not, folding in what this step built.\n")
        }
    }
}

/// Where a claim on a flow sent the work, and the line that starts it.
fn describe_route(anchor: &str, route: &FlowRoute) -> String {
    match route {
        FlowRoute::Followed { outcome, next } => format!(
            "\nRouted by `{outcome}` to step {number} \"{}\", now open and current (visit {}). Start it now with \
             `/rk {anchor} step:{number} start`, without waiting for anyone.",
            next.title,
            next.visits.len(),
            number = next.position + 1
        ),
        FlowRoute::Ended { outcome: Some(outcome) } => {
            format!("\nRouted by `{outcome}` to the end of the flow: it is finished bar the review.")
        }
        FlowRoute::Ended { outcome: None } => "\nThat step has no routes out: the flow is finished bar the review.".to_string(),
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
                "outcome",
                "Only with `claimed`, on a task whose steps carry routes: what happened, as the route names it \
                 (`passed`, `app-error`). Rekall follows that route to the next step. Not needed when the step has \
                 one route or none.",
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
        let outcome = args.optional_string("outcome");
        let requested = parse_state(&raw_state)?;

        let flow_move = self
            .flow
            .transition(target.project_label.as_deref(), &target.task_label, Some(&step_ref), requested, outcome.as_deref())
            .await
            .map_err(|e| told(e, QUALIFY_WITH_PROJECT))?;

        let moved = &flow_move.step;
        let committed = if moved.state == TaskStepState::Claimed {
            Some(self.auto_commit.after_step_claim(moved.task_id, moved.id, commit_message.as_deref()).await?)
        } else {
            None
        };
        self.report(&target, &flow_move, &raw_state, committed.as_ref()).await
    }
}
