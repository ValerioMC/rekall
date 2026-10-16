use async_trait::async_trait;
use rekall_service::section::{ProjectSectionService, SectionsRead};
use serde_json::Value;

use crate::tool::{told, Anchor, AnchoredTask, Arguments, QUALIFY_WITH_PROJECT};
use crate::protocol::{McpTool, ToolError, ToolSchema};
use crate::texts;

/// Lets a session read a project's sections, which no context carries. Without a title it lists
/// them, with one it returns that section whole; it writes nothing.
pub struct SectionReadTool {
    sections: ProjectSectionService,
}

impl SectionReadTool {
    pub fn new(sections: ProjectSectionService) -> Self {
        Self { sections }
    }
}

/// Where to read: a project alone, or a task whose project it is.
struct ReadTarget {
    project_label: Option<String>,
    task_label: Option<String>,
}

#[async_trait]
impl McpTool for SectionReadTool {
    fn name(&self) -> &'static str {
        "rekall_read_sections"
    }

    fn description(&self) -> &'static str {
        texts::READ_SECTIONS
    }

    fn input_schema(&self) -> Value {
        ToolSchema::object()
            .required_string(
                "anchors",
                "The project, `project:vega`, or a task and its project, `project:vega task:report-builder`. Labels, \
                 never titles.",
            )
            .optional_string(
                "title",
                "The section to read, compared ignoring case. Omitted, the project's sections are listed by title.",
            )
            .build()
    }

    async fn execute(&self, arguments: Option<&Value>) -> Result<String, ToolError> {
        let args = Arguments::of(arguments);
        let target = target_of(&Anchor::parse_all(Some(&args.required_string("anchors")?))?)?;
        let hint = if target.task_label.is_some() { QUALIFY_WITH_PROJECT } else { "" };
        let read = self
            .sections
            .read(target.project_label.as_deref(), target.task_label.as_deref(), args.optional_string("title").as_deref())
            .await
            .map_err(|e| told(e, hint))?;
        Ok(report(&read))
    }
}

fn target_of(anchors: &[Anchor]) -> Result<ReadTarget, ToolError> {
    if anchors.iter().any(|anchor| anchor.is("task") || !anchor.is_qualified()) {
        let task = AnchoredTask::from(anchors)?;
        return Ok(ReadTarget { project_label: task.project_label, task_label: Some(task.task_label) });
    }
    match anchors {
        [project] if project.is("project") => {
            Ok(ReadTarget { project_label: Some(project.value.clone()), task_label: None })
        }
        _ => Err(ToolError::Failure(
            "Sections belong to one project: pass `project:<label>` or `project:<label> task:<label>`.".into(),
        )),
    }
}

fn report(read: &SectionsRead) -> String {
    if read.asked_for_one {
        return read
            .sections
            .iter()
            .map(|section| format!("Section \"{}\" of `{}`:\n\n{}", section.title, read.project_anchor, section.body_markdown))
            .collect();
    }
    if read.sections.is_empty() {
        return format!("`{}` has no sections.", read.project_anchor);
    }
    let lines: Vec<String> = read
        .sections
        .iter()
        .map(|section| format!("- {} ({} characters)", section.title, section.body_markdown.chars().count()))
        .collect();
    format!(
        "`{}` has {} section{}. Pass `title` to read one:\n{}",
        read.project_anchor,
        read.sections.len(),
        if read.sections.len() == 1 { "" } else { "s" },
        lines.join("\n")
    )
}

#[cfg(test)]
#[path = "../../tests/unit/section/section_read_tool_tests.rs"]
mod tests;
