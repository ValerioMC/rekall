use async_trait::async_trait;
use rekall_common::Id;
use rekall_service::diagram::{DiagramService, DiagramTarget, GeneratedDiagram, GeneratedDiagramView};
use serde_json::{json, Value};

use super::{told, Anchor, AnchoredTask, Arguments, QUALIFY_WITH_PROJECT};
use crate::protocol::{McpTool, ToolError, ToolSchema};
use crate::texts;

/// Stores the diagram a session generated for a project or a task. It writes a diagram whole, new
/// or in place of one it names, and never decides how it is drawn: that is the console's.
pub struct DiagramTool {
    diagrams: DiagramService,
}

impl DiagramTool {
    pub fn new(diagrams: DiagramService) -> Self {
        Self { diagrams }
    }
}

#[async_trait]
impl McpTool for DiagramTool {
    fn name(&self) -> &'static str {
        "rekall_diagram"
    }

    fn writes(&self) -> bool {
        true
    }

    fn description(&self) -> &'static str {
        texts::DIAGRAM
    }

    fn input_schema(&self) -> Value {
        ToolSchema::object()
            .required_string(
                "anchors",
                "The project the diagram is for, `project:vega`, or a task and its project, `project:vega task:report-builder`. \
                 Labels, never titles.",
            )
            .required_string("title", "What the diagram shows, short, under 200 characters.")
            .required_string("question", "The request the diagram answers, in the words it was asked.")
            .required_object_array(
                "sheets",
                "One to eight sheets: `[{\"title\":\"Checkout\",\"mermaid\":\"sequenceDiagram\\n…\",\"links\":{\"S\":[{\"file\":\"src/order.rs\",\"startLine\":10,\"endLine\":80}]}}]`.",
            )
            .optional_string(
                "diagram",
                "The id of a diagram these sheets replace, as an earlier answer gave it. Omitted, a new diagram is created.",
            )
            .build()
    }

    async fn execute(&self, arguments: Option<&Value>) -> Result<String, ToolError> {
        let args = Arguments::of(arguments);
        let target = target_of(&Anchor::parse_all(Some(&args.required_string("anchors")?))?)?;
        let document = json!({ "sheets": sheets_argument(args.value("sheets"))? });
        let generated = GeneratedDiagram {
            target,
            diagram_id: diagram_id(args.optional_string("diagram"))?,
            title: args.required_string("title")?,
            question: args.required_string("question")?,
            document: DiagramService::read_document(document).map_err(|e| told(e, ""))?,
        };
        let written = self.diagrams.write_generated(generated).await.map_err(|e| told(e, QUALIFY_WITH_PROJECT))?;
        Ok(written_report(&written))
    }
}

/// A project alone, or a task with the project that disambiguates it.
fn target_of(anchors: &[Anchor]) -> Result<DiagramTarget, ToolError> {
    if anchors.iter().any(|anchor| anchor.is("task") || !anchor.is_qualified()) {
        let task = AnchoredTask::from(anchors)?;
        return Ok(DiagramTarget::Task { project_label: task.project_label, task_label: task.task_label });
    }
    match anchors {
        [project] if project.is("project") => Ok(DiagramTarget::Project { label: project.value.clone() }),
        _ => Err(ToolError::Failure(
            "A diagram belongs to one project, and optionally one of its tasks: pass `project:<label>` or \
             `project:<label> task:<label>`."
                .into(),
        )),
    }
}

/// The sheets as an array, or as a string holding one: some clients serialise a nested argument.
fn sheets_argument(argument: Option<&Value>) -> Result<Value, ToolError> {
    match argument {
        None => Err(ToolError::Illegal("'sheets' is required".into())),
        Some(Value::String(text)) => serde_json::from_str(text)
            .map_err(|error| ToolError::Illegal(format!("'sheets' is a string that is not a JSON array: {error}"))),
        Some(sheets) => Ok(sheets.clone()),
    }
}

fn diagram_id(reference: Option<String>) -> Result<Option<Id>, ToolError> {
    reference
        .map(|text| {
            text.trim().parse::<Id>().map_err(|_| {
                ToolError::Illegal(format!("'diagram' is {text:?}, not a diagram id. Leave it out to create a new diagram."))
            })
        })
        .transpose()
}

fn written_report(written: &GeneratedDiagramView) -> String {
    let summary = &written.diagram.summary;
    let kinds: Vec<&str> = summary.kinds.iter().map(|kind| kind.name()).collect();
    let linked: usize = written.diagram.sheets.iter().map(|sheet| sheet.links.len()).sum();
    let mut out = format!(
        "Diagram \"{}\" {} on `{}` as `{}`: {} sheet{} ({}), {linked} element{} linked to code. The console's Diagrams \
         screen shows it now. To revise it, send every sheet again with `diagram` set to `{}`, keeping the ids of the \
         elements that stay.",
        summary.title,
        if written.replaced { "replaced" } else { "written" },
        written.anchors,
        summary.id,
        kinds.len(),
        plural(kinds.len()),
        kinds.join(", "),
        plural(linked),
        summary.id
    );
    if !written.sources_checked {
        out.push_str(
            " The project has no folder, so its links were not held against the code and the console cannot open \
             them until one is set on the project page.",
        );
    }
    out
}

fn plural(count: usize) -> &'static str {
    if count == 1 {
        ""
    } else {
        "s"
    }
}

#[cfg(test)]
#[path = "../../tests/unit/tool/diagram_tool_tests.rs"]
mod tests;
