use async_trait::async_trait;
use rekall_core::flow::FlowProposalService;
use rekall_core::step::Staging;
use serde_json::Value;

use super::plan_of;
use crate::protocol::{McpTool, ToolError, ToolSchema};
use crate::texts;
use crate::tool::{told, Anchor, AnchoredTask, Arguments, QUALIFY_WITH_PROJECT};

/// Lets a session redesign a task's steps: it deletes the draft and open ones and lays the new
/// plan in their place, as a checklist or a flow, in one transaction. Started, claimed and done
/// steps stay.
pub struct ReplanTool {
    proposals: FlowProposalService,
}

impl ReplanTool {
    pub fn new(proposals: FlowProposalService) -> Self {
        Self { proposals }
    }
}

#[async_trait]
impl McpTool for ReplanTool {
    fn name(&self) -> &'static str {
        "rekall_replan"
    }

    fn writes(&self) -> bool {
        true
    }

    fn description(&self) -> &'static str {
        texts::REPLAN
    }

    fn input_schema(&self) -> Value {
        ToolSchema::object()
            .required_string(
                "anchors",
                "The task to replan, e.g. `project:vega task:release`. Labels, never titles, and it has to name exactly one task.",
            )
            .required_object_array(
                "steps",
                "The new steps in order, as for `rekall_propose_flow`: `[{\"title\":\"Run the tests\",\"detail\":\"markdown\",\
                 \"new_session\":true,\"max_visits\":3,\"routes\":[{\"outcome\":\"passed\",\"to\":\"end\"}]}]`. Without any `routes` \
                 they are a checklist, in the order listed; with routes they are a flow.",
            )
            .optional_boolean(
                "draft",
                "True lays the new steps as drafts for a person to promote; left out, they are ready to work.",
            )
            .build()
    }

    async fn execute(&self, arguments: Option<&Value>) -> Result<String, ToolError> {
        let args = Arguments::of(arguments);
        let target = AnchoredTask::from(&Anchor::parse_all(Some(&args.required_string("anchors")?))?)?;
        let plan = plan_of(args.value("steps"))?;
        let staging = if args.flag("draft") { Staging::Draft } else { Staging::Ready };
        let replanned = self
            .proposals
            .replan(target.project_label.as_deref(), &target.task_label, &plan, staging)
            .await
            .map_err(|e| told(e, QUALIFY_WITH_PROJECT))?;
        let state = if staging == Staging::Ready { texts::REPLANNED_READY } else { texts::REPLANNED_DRAFT };
        Ok(texts::REPLANNED_REPORT
            .replacen("%d", &replanned.removed.to_string(), 1)
            .replacen("%s", plural(replanned.removed), 1)
            .replacen("%d", &replanned.laid.steps.to_string(), 1)
            .replacen("%s", plural(replanned.laid.steps), 1)
            .replacen("%d", &replanned.laid.routes.to_string(), 1)
            .replacen("%s", plural(replanned.laid.routes), 1)
            .replacen("%s", state, 1)
            .replacen("%d", &replanned.kept.to_string(), 1)
            .replacen("%s", plural(replanned.kept), 1))
    }
}

fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}
