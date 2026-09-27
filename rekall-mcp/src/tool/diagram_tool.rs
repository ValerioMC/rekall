use async_trait::async_trait;
use rekall_common::Id;
use rekall_service::diagram::{DiagramService, GeneratedDiagram, GeneratedDiagramView};
use rekall_service::Services;
use serde_json::Value;

use super::{told, Anchor, AnchoredTask, Arguments, QUALIFY_WITH_PROJECT};
use crate::protocol::{McpTool, ToolError, ToolSchema};
use crate::texts;

/// Stores the Semantic Graph a session generated for a task. It writes a graph whole, new or in
/// place of one it names, and never decides how the graph is drawn: that is the console's.
pub struct DiagramTool {
    services: Services,
}

impl DiagramTool {
    pub fn new(services: Services) -> Self {
        Self { services }
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
                "The task the diagram is generated for, e.g. `project:vega task:report-builder`. Labels, never titles, \
                 and it has to name exactly one task.",
            )
            .required_string("title", "What the diagram shows, short, under 200 characters.")
            .required_string("question", "The request the diagram answers, in the words it was asked.")
            .required_object(
                "graph",
                "The whole Semantic Graph document: `{\"format\":\"rekall.semantic-graph\",\"version\":1,\"nodes\":[…],\"edges\":[…]}`.",
            )
            .optional_string(
                "diagram",
                "The id of a diagram this graph replaces, as an earlier answer gave it. Omitted, a new diagram is created.",
            )
            .build()
    }

    async fn execute(&self, arguments: Option<&Value>) -> Result<String, ToolError> {
        let args = Arguments::of(arguments);
        let target = AnchoredTask::from(&Anchor::parse_all(Some(&args.required_string("anchors")?))?)?;
        let generated = GeneratedDiagram {
            project_label: target.project_label,
            task_label: target.task_label,
            diagram_id: diagram_id(args.optional_string("diagram"))?,
            title: args.required_string("title")?,
            question: args.required_string("question")?,
            graph: DiagramService::read_graph(graph_document(args.value("graph"))?).map_err(|e| told(e, ""))?,
        };
        let written = self.services.diagrams.write_generated(generated).await.map_err(|e| told(e, QUALIFY_WITH_PROJECT))?;
        Ok(written_report(&written))
    }
}

/// The graph as an object, or as a string holding one: some clients serialise a nested argument.
fn graph_document(argument: Option<&Value>) -> Result<Value, ToolError> {
    match argument {
        None => Err(ToolError::Illegal("'graph' is required".into())),
        Some(Value::String(text)) => serde_json::from_str(text)
            .map_err(|error| ToolError::Illegal(format!("'graph' is a string that is not a JSON document: {error}"))),
        Some(document) => Ok(document.clone()),
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
    let traced = written.diagram.graph.nodes.iter().filter(|node| !node.sources.is_empty()).count();
    let mut out = format!(
        "Diagram \"{}\" {} on `{}` as `{}`: {} node{} ({traced} traced to code) and {} edge{}. The console's \
         Diagrams screen shows it now. To revise it, send the whole graph again with `diagram` set to `{}`, keeping \
         the ids of the elements that stay.",
        summary.title,
        if written.replaced { "replaced" } else { "written" },
        written.task_anchor,
        summary.id,
        summary.node_count,
        plural(summary.node_count),
        summary.edge_count,
        plural(summary.edge_count),
        summary.id
    );
    if !written.sources_checked {
        out.push_str(
            " The project has no folder, so its spans were not held against the code and the console cannot open \
             them until one is set on the project page.",
        );
    }
    out
}

fn plural(count: i32) -> &'static str {
    if count == 1 {
        ""
    } else {
        "s"
    }
}

#[cfg(test)]
#[path = "../../tests/unit/tool/diagram_tool_tests.rs"]
mod tests;
