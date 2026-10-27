use async_trait::async_trait;
use rekall_service::task::{EpicJoined, EpicMembershipService};
use serde_json::Value;

use crate::protocol::{McpTool, ToolError, ToolSchema};
use crate::texts;
use crate::tool::{told, Anchor, AnchoredTask, Arguments, QUALIFY_WITH_PROJECT};

/// Lets a session file a task under an epic of its project: it moves a task into an epic, nothing else.
pub struct EpicMembershipTool {
    membership: EpicMembershipService,
}

impl EpicMembershipTool {
    pub fn new(membership: EpicMembershipService) -> Self {
        Self { membership }
    }
}

#[async_trait]
impl McpTool for EpicMembershipTool {
    fn name(&self) -> &'static str {
        "rekall_join_epic"
    }

    fn writes(&self) -> bool {
        true
    }

    fn description(&self) -> &'static str {
        texts::JOIN_EPIC
    }

    fn input_schema(&self) -> Value {
        ToolSchema::object()
            .required_string(
                "anchors",
                "The task to file, e.g. `project:vega task:pay`. Labels, never titles, and it has to name exactly one task.",
            )
            .required_string("epic", "The label of an epic of the same project, `checkout`.")
            .build()
    }

    async fn execute(&self, arguments: Option<&Value>) -> Result<String, ToolError> {
        let args = Arguments::of(arguments);
        let target = AnchoredTask::from(&Anchor::parse_all(Some(&args.required_string("anchors")?))?)?;
        let epic = args.required_string("epic")?;
        let joined = self
            .membership
            .join(target.project_label.as_deref(), &target.task_label, &epic)
            .await
            .map_err(|e| told(e, QUALIFY_WITH_PROJECT))?;
        Ok(report(&joined))
    }
}

fn report(joined: &EpicJoined) -> String {
    let left = joined.left.as_ref().map(|left| format!(", out of `task:{left}`")).unwrap_or_default();
    texts::EPIC_JOINED_REPORT
        .replacen("%s", &joined.task_anchor, 1)
        .replacen("%s", &joined.epic_title, 1)
        .replacen("%s", &joined.epic_anchor, 1)
        .replacen("%s", &left, 1)
}

#[cfg(test)]
#[path = "../../tests/unit/task/epic_membership_tool_tests.rs"]
mod tests;
