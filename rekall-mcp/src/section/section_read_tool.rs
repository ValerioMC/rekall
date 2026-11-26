use async_trait::async_trait;
use rekall_core::context::SessionPictures;
use rekall_core::section::{ProjectSectionService, ProjectSectionView, SectionsRead};
use rekall_core::ProjectRef;
use serde_json::Value;

use crate::tool::{told, Anchor, AnchoredProject, AnchoredTask, Arguments, QUALIFY_WITH_PROJECT};
use crate::protocol::{McpTool, ToolError, ToolSchema};
use crate::texts;

/// Lets a session read a project's sections, which no context carries. Without a title it lists
/// them, with one it returns that section whole; it writes nothing.
pub struct SectionReadTool {
    sections: ProjectSectionService,
    pictures: SessionPictures,
}

impl SectionReadTool {
    pub fn new(sections: ProjectSectionService, pictures: SessionPictures) -> Self {
        Self { sections, pictures }
    }
}

/// Where to read: a project alone, or a task whose project it is.
pub(super) struct ReadTarget {
    pub project: Option<ProjectRef>,
    pub task_code: Option<String>,
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
                "The project, `project:vega`, or a task and its project, `task:RK-12`. Labels, \
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
        let hint = if target.task_code.is_some() { QUALIFY_WITH_PROJECT } else { "" };
        let read = self
            .sections
            .read(target.project.as_ref(), target.task_code.as_deref(), args.optional_string("title").as_deref())
            .await
            .map_err(|e| told(e, hint))?;
        Ok(self.pictures.appended_to(report(&read)))
    }
}

pub(super) fn target_of(anchors: &[Anchor]) -> Result<ReadTarget, ToolError> {
    if anchors.iter().any(|anchor| anchor.is("task") || !anchor.is_qualified()) {
        let task = AnchoredTask::from(anchors)?;
        return Ok(ReadTarget { project: task.project_label.as_deref().map(ProjectRef::bare), task_code: Some(task.task_code) });
    }
    match AnchoredProject::alone(anchors) {
        Some(project) => Ok(ReadTarget { project: Some(project), task_code: None }),
        None => Err(ToolError::Failure(
            "Sections belong to one project: pass `project:<label>` or `project:<label> task:<code>`, with \
             `company:<name>` beside the label when two companies share it."
                .into(),
        )),
    }
}

fn report(read: &SectionsRead) -> String {
    if read.asked_for_one {
        return read
            .sections
            .iter()
            .map(|section| {
                let context = context_title(read, section);
                format!("Section \"{}\" of `{}`, in the context \"{context}\":\n\n{}", section.title, read.project_anchor, section.body_markdown)
            })
            .collect();
    }
    if read.sections.is_empty() {
        return format!("`{}` has no sections.", read.project_anchor);
    }
    let groups: Vec<String> = read
        .contexts
        .iter()
        .filter_map(|context| {
            let lines: Vec<String> = read
                .sections
                .iter()
                .filter(|section| section.context_id == context.id)
                .map(|section| format!("- {} ({} characters)", section.title, section.body_markdown.chars().count()))
                .collect();
            (!lines.is_empty()).then(|| format!("{}:\n{}", context.title, lines.join("\n")))
        })
        .collect();
    format!(
        "`{}` has {} section{}. Pass `title` to read one:\n\n{}",
        read.project_anchor,
        read.sections.len(),
        if read.sections.len() == 1 { "" } else { "s" },
        groups.join("\n\n")
    )
}

fn context_title<'a>(read: &'a SectionsRead, section: &ProjectSectionView) -> &'a str {
    read.contexts.iter().find(|context| context.id == section.context_id).map(|context| context.title.as_str()).unwrap_or_default()
}

#[cfg(test)]
#[path = "../../tests/unit/section/section_read_tool_tests.rs"]
mod tests;
