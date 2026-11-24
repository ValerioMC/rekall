use async_trait::async_trait;
use rekall_core::section::{PageGroup, ProjectSectionService, SectionsReorganized};
use serde::Deserialize;
use serde_json::Value;

use super::section_read_tool::target_of;
use crate::protocol::{McpTool, ToolError, ToolSchema};
use crate::texts;
use crate::tool::{told, Anchor, Arguments, QUALIFY_WITH_PROJECT};

/// Lets a session regroup a project's pages by concept: every page named once under the contexts it
/// wants. Pages only move; none is renamed, rewritten or deleted.
pub struct SectionReorganizeTool {
    sections: ProjectSectionService,
}

impl SectionReorganizeTool {
    pub fn new(sections: ProjectSectionService) -> Self {
        Self { sections }
    }
}

#[derive(Deserialize)]
struct GroupArgument {
    context: String,
    pages: Vec<String>,
}

#[async_trait]
impl McpTool for SectionReorganizeTool {
    fn name(&self) -> &'static str {
        "rekall_reorganize_sections"
    }

    fn writes(&self) -> bool {
        true
    }

    fn description(&self) -> &'static str {
        texts::REORGANIZE_SECTIONS
    }

    fn input_schema(&self) -> Value {
        ToolSchema::object()
            .required_string("anchors", "The project, `project:vega`, or a task and its project, `task:RK-12`. Labels, never titles.")
            .required_object_array(
                "groups",
                "The whole arrangement, contexts in the order to be shown: `[{\"context\":\"Operations\",\"pages\":[\"Deployment\",\"On-call runbook\"]}]`. \
                 Every page of the project is named exactly once, by title.",
            )
            .build()
    }

    async fn execute(&self, arguments: Option<&Value>) -> Result<String, ToolError> {
        let args = Arguments::of(arguments);
        let target = target_of(&Anchor::parse_all(Some(&args.required_string("anchors")?))?)?;
        let groups = groups_of(args.value("groups"))?;
        let hint = if target.task_code.is_some() { QUALIFY_WITH_PROJECT } else { "" };
        let done = self
            .sections
            .reorganize(target.project.as_ref(), target.task_code.as_deref(), &groups)
            .await
            .map_err(|e| told(e, hint))?;
        Ok(report(&done))
    }
}

fn groups_of(argument: Option<&Value>) -> Result<Vec<PageGroup>, ToolError> {
    let groups: Vec<GroupArgument> = match argument {
        None => return Err(ToolError::Illegal("A reorganization needs its `groups`.".into())),
        Some(Value::String(text)) => serde_json::from_str(text),
        Some(array) => serde_json::from_value(array.clone()),
    }
    .map_err(|error| ToolError::Illegal(format!("'groups' is not a list of {{context, pages}} objects: {error}")))?;
    Ok(groups.into_iter().map(|group| PageGroup { context: group.context, pages: group.pages }).collect())
}

fn report(done: &SectionsReorganized) -> String {
    let lines: Vec<String> = done
        .contexts
        .iter()
        .map(|(title, pages)| format!("- {title}: {pages} page{}", if *pages == 1 { "" } else { "s" }))
        .collect();
    let dropped = match done.contexts_dropped {
        0 => String::new(),
        1 => " One emptied context was dropped.".to_string(),
        count => format!(" {count} emptied contexts were dropped."),
    };
    format!(
        "`{}` reorganized: {} page{} changed context.{dropped}\n{}",
        done.project_anchor,
        done.pages_moved,
        if done.pages_moved == 1 { "" } else { "s" },
        lines.join("\n")
    )
}
