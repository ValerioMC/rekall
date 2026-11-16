use async_trait::async_trait;
use rekall_core::component::{ComponentWritten, ProjectComponentService};
use serde_json::Value;

use crate::protocol::{McpTool, ToolError, ToolSchema};
use crate::texts;
use crate::tool::{told, Anchor, AnchoredProject, Arguments};

/// Lets a session record a folder of a project as a component: a name, a folder, a summary that
/// every context carries and a structured description read on demand. The component with that name
/// or folder is rewritten; nothing can be reordered or deleted.
pub struct ComponentTool {
    components: ProjectComponentService,
}

impl ComponentTool {
    pub fn new(components: ProjectComponentService) -> Self {
        Self { components }
    }
}

#[async_trait]
impl McpTool for ComponentTool {
    fn name(&self) -> &'static str {
        "rekall_component"
    }

    fn writes(&self) -> bool {
        true
    }

    fn description(&self) -> &'static str {
        texts::COMPONENT
    }

    fn input_schema(&self) -> Value {
        ToolSchema::object()
            .required_string("anchors", "The project the component belongs to, `project:vega`. A label, never a title.")
            .required_string("name", "What the folder is called in the project: the service, module or app, short.")
            .required_string("folder", "The folder, as an absolute path or relative to the project's folder.")
            .required_string("summary", "What the component is for, in two or three sentences and 500 characters at most. Every context carries it.")
            .required_string("description", "The structured description in markdown, with the headings the playbook lists, tables and Mermaid diagrams where they say more than prose.")
            .build()
    }

    async fn execute(&self, arguments: Option<&Value>) -> Result<String, ToolError> {
        let args = Arguments::of(arguments);
        let target = AnchoredProject::from(&Anchor::parse_all(Some(&args.required_string("anchors")?))?)?;
        let draft = rekall_core::component::ComponentDraft {
            name: Some(&args.required_string("name")?),
            folder: Some(&args.required_string("folder")?),
            summary: Some(&args.required_string("summary")?),
            description: Some(&args.required_string("description")?),
        };
        let written = self.components.write(&target.project, draft).await.map_err(|e| told(e, ""))?;
        Ok(report(&written))
    }
}

fn report(written: &ComponentWritten) -> String {
    let count = written.components_on_project;
    let plural = if count == 1 { "" } else { "s" };
    if written.replaced {
        return format!(
            "Component \"{}\" on `{}` rewritten. Its earlier text is gone: nothing keeps a component's history. The project has {count} component{plural}.",
            written.name, written.project_anchor
        );
    }
    format!(
        "Component \"{}\" written on `{}`. The project has {count} component{plural}: every context lists each one's summary, and the console shows the whole description.",
        written.name, written.project_anchor
    )
}

#[cfg(test)]
#[path = "../../tests/unit/component/component_tool_tests.rs"]
mod tests;
