use async_trait::async_trait;
use rekall_core::flow::{FlowPlan, FlowProposalService, PlannedRoute, PlannedStep};
use rekall_core::step::VisitBudget;
use serde::Deserialize;
use serde_json::Value;

use crate::protocol::{McpTool, ToolError, ToolSchema};
use crate::texts;
use crate::tool::{told, Anchor, AnchoredTask, Arguments, QUALIFY_WITH_PROJECT};

/// Lets a planning session propose a flow whole: its steps as drafts and the routes between them.
/// Nothing it writes is work, and no route of a released step changes, until a person promotes.
pub struct FlowProposalTool {
    proposals: FlowProposalService,
}

impl FlowProposalTool {
    pub fn new(proposals: FlowProposalService) -> Self {
        Self { proposals }
    }
}

#[derive(Deserialize)]
struct StepArgument {
    title: String,
    detail: Option<String>,
    #[serde(default)]
    routes: Vec<RouteArgument>,
    max_visits: Option<i32>,
    #[serde(default)]
    new_session: bool,
}

#[derive(Deserialize)]
struct RouteArgument {
    outcome: String,
    to: TargetArgument,
}

/// A step number in the flow, or `end`; some clients send the number as a string.
#[derive(Deserialize)]
#[serde(untagged)]
enum TargetArgument {
    Number(usize),
    Word(String),
}

#[async_trait]
impl McpTool for FlowProposalTool {
    fn name(&self) -> &'static str {
        "rekall_propose_flow"
    }

    fn writes(&self) -> bool {
        true
    }

    fn description(&self) -> &'static str {
        texts::PROPOSE_FLOW
    }

    fn input_schema(&self) -> Value {
        ToolSchema::object()
            .required_string(
                "anchors",
                "The task to propose the flow for, e.g. `project:vega task:release`. Labels, never titles, and it has \
                 to name exactly one task.",
            )
            .required_object_array(
                "steps",
                "The flow's steps in order, the first where it begins: `[{\"title\":\"Run the tests\",\"detail\":\"markdown\",\
                 \"max_visits\":3,\"routes\":[{\"outcome\":\"passed\",\"to\":\"end\"},{\"outcome\":\"app-error\",\"to\":3}]}]`. \
                 `to` is a step's one-based number in this list, or `end`; `max_visits` (1 to 99) and `new_session` (true opens a fresh session) are optional.",
            )
            .build()
    }

    async fn execute(&self, arguments: Option<&Value>) -> Result<String, ToolError> {
        let args = Arguments::of(arguments);
        let target = AnchoredTask::from(&Anchor::parse_all(Some(&args.required_string("anchors")?))?)?;
        let plan = plan_of(args.value("steps"))?;
        let proposed = self
            .proposals
            .propose(target.project_label.as_deref(), &target.task_label, &plan)
            .await
            .map_err(|e| told(e, QUALIFY_WITH_PROJECT))?;
        Ok(texts::PROPOSED_FLOW_REPORT
            .replacen("%d", &proposed.steps.to_string(), 1)
            .replacen("%s", if proposed.steps == 1 { "" } else { "s" }, 1)
            .replacen("%d", &proposed.routes.to_string(), 1)
            .replacen("%s", if proposed.routes == 1 { "" } else { "s" }, 1))
    }
}

/// The steps as an array, or as a string holding one: some clients serialise a nested argument.
fn plan_of(argument: Option<&Value>) -> Result<FlowPlan, ToolError> {
    let steps: Vec<StepArgument> = match argument {
        None => return Err(ToolError::Illegal("A flow needs its `steps`.".into())),
        Some(Value::String(text)) => serde_json::from_str(text),
        Some(array) => serde_json::from_value(array.clone()),
    }
    .map_err(|error| {
        ToolError::Illegal(format!(
            "'steps' is not a list of {{title, detail, routes: [{{outcome, to}}]}} objects: {error}"
        ))
    })?;
    let planned = steps.into_iter().map(planned_step).collect::<Result<Vec<_>, _>>()?;
    Ok(FlowPlan::new(planned))
}

fn planned_step(step: StepArgument) -> Result<PlannedStep, ToolError> {
    let routes = step
        .routes
        .into_iter()
        .map(|route| Ok(PlannedRoute { to: target_index(&step.title, &route.to)?, outcome: route.outcome }))
        .collect::<Result<Vec<_>, ToolError>>()?;
    let max_visits = step.max_visits.map(VisitBudget::new).transpose().map_err(|e| told(e, ""))?;
    Ok(PlannedStep { title: step.title, detail: step.detail, routes, max_visits, starts_session: step.new_session })
}

/// The zero-based step a route's `to` names, or `None` for the end of the flow.
fn target_index(step_title: &str, target: &TargetArgument) -> Result<Option<usize>, ToolError> {
    let number = match target {
        TargetArgument::Number(number) => Some(*number),
        TargetArgument::Word(word) if word.trim().eq_ignore_ascii_case("end") => return Ok(None),
        TargetArgument::Word(word) => word.trim().parse::<usize>().ok(),
    };
    match number {
        Some(number) if number >= 1 => Ok(Some(number - 1)),
        _ => Err(ToolError::Illegal(format!(
            "A route of '{step_title}' leads nowhere: `to` is a step's one-based number in this flow, or `end`."
        ))),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/flow/flow_proposal_tool_tests.rs"]
mod tests;
