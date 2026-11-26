use async_trait::async_trait;
use rekall_core::component::{ComponentsRead, ProjectComponentService};
use rekall_core::context::SessionPictures;
use serde_json::Value;

use crate::protocol::{McpTool, ToolError, ToolSchema};
use crate::texts;
use crate::tool::{told, Anchor, AnchoredProject, Arguments};

/// Lets a session read a project's components whole, past the summaries a context carries. Without
/// a name it lists them, with one it returns that component's description; it writes nothing.
pub struct ComponentReadTool {
    components: ProjectComponentService,
    pictures: SessionPictures,
}

impl ComponentReadTool {
    pub fn new(components: ProjectComponentService, pictures: SessionPictures) -> Self {
        Self { components, pictures }
    }
}

#[async_trait]
impl McpTool for ComponentReadTool {
    fn name(&self) -> &'static str {
        "rekall_read_components"
    }

    fn description(&self) -> &'static str {
        texts::READ_COMPONENTS
    }

    fn input_schema(&self) -> Value {
        ToolSchema::object()
            .required_string("anchors", "The project, `project:vega`. A label, never a title.")
            .optional_string("name", "The component to read, compared ignoring case. Omitted, the project's components are listed with their summaries.")
            .build()
    }

    async fn execute(&self, arguments: Option<&Value>) -> Result<String, ToolError> {
        let args = Arguments::of(arguments);
        let target = AnchoredProject::from(&Anchor::parse_all(Some(&args.required_string("anchors")?))?)?;
        let read = self
            .components
            .read(&target.project, args.optional_string("name").as_deref())
            .await
            .map_err(|e| told(e, ""))?;
        Ok(self.pictures.appended_to(report(&read)))
    }
}

fn report(read: &ComponentsRead) -> String {
    if read.asked_for_one {
        return read
            .components
            .iter()
            .map(|component| {
                format!("Component \"{}\" of `{}`, folder `{}`:\n\n{}\n\n{}", component.name, read.project_anchor, component.folder, component.summary, component.description)
            })
            .collect();
    }
    if read.components.is_empty() {
        return format!("`{}` has no components.", read.project_anchor);
    }
    let lines: Vec<String> = read
        .components
        .iter()
        .map(|component| format!("- {} (`{}`, {} characters): {}", component.name, component.folder, component.description.chars().count(), component.summary))
        .collect();
    format!(
        "`{}` has {} component{}. Pass `name` to read one whole:\n{}",
        read.project_anchor,
        read.components.len(),
        if read.components.len() == 1 { "" } else { "s" },
        lines.join("\n")
    )
}

#[cfg(test)]
#[path = "../../tests/unit/component/component_read_tool_tests.rs"]
mod tests;
