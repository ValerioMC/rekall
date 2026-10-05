use async_trait::async_trait;
use rekall_service::note::OnTitleClash;
use rekall_service::Services;
use serde_json::Value;

use super::{told, Anchor, AnchoredTask, Arguments, QUALIFY_WITH_PROJECT};
use crate::protocol::{McpTool, ToolError, ToolSchema};
use crate::texts;

/// Lets a session keep project-wide knowledge in a titled section of the task's project, out of
/// every context. It adds a section, or with `replace` rewrites the body of the one with that
/// title; nothing can be renamed, reordered or deleted.
pub struct ProjectSectionTool {
    services: Services,
}

impl ProjectSectionTool {
    pub fn new(services: Services) -> Self {
        Self { services }
    }
}

#[async_trait]
impl McpTool for ProjectSectionTool {
    fn name(&self) -> &'static str {
        "rekall_project_section"
    }

    fn writes(&self) -> bool {
        true
    }

    fn description(&self) -> &'static str {
        texts::PROJECT_SECTION
    }

    fn input_schema(&self) -> Value {
        ToolSchema::object()
            .required_string(
                "anchors",
                "The task being worked, e.g. `project:vega task:report-builder`. The section lands on its project. \
                 Labels, never titles, and it has to name exactly one task.",
            )
            .required_string("title", "What the section is, short, like a heading.")
            .required_string("body", "The complete section, in markdown.")
            .optional_boolean(
                "replace",
                "`true` rewrites the body of the section this project already has under the same title (compared \
                 ignoring case), or adds it if there is none. Omitted, a title the project already uses is refused.",
            )
            .build()
    }

    async fn execute(&self, arguments: Option<&Value>) -> Result<String, ToolError> {
        let args = Arguments::of(arguments);
        let target = AnchoredTask::from(&Anchor::parse_all(Some(&args.required_string("anchors")?))?)?;
        let title = args.required_string("title")?;
        let body = args.required_string("body")?;
        let on_clash = if args.flag("replace") { OnTitleClash::Replace } else { OnTitleClash::Refuse };
        let written = self
            .services
            .sections
            .write(target.project_label.as_deref(), &target.task_label, Some(&title), Some(&body), on_clash)
            .await
            .map_err(|e| told(e, QUALIFY_WITH_PROJECT))?;
        let count = written.sections_on_project;
        let plural = if count == 1 { "" } else { "s" };
        if written.replaced {
            return Ok(format!(
                "Section \"{}\" on `{}` rewritten. Its earlier body is gone: no history keeps a section's text. The \
                 project has {count} section{plural}.",
                written.title, written.project_anchor
            ));
        }
        Ok(format!(
            "Section \"{}\" written on `{}`. The project has {count} section{plural}, shown beside its blueprint in \
             the console and loaded into no context.",
            written.title, written.project_anchor
        ))
    }
}
