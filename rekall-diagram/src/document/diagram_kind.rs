use serde::{Deserialize, Serialize};

/// The Mermaid diagram types a sheet can declare. The console draws the five that link elements
/// to code with its own components, and hands the rest to Mermaid's renderer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagramKind {
    Sequence,
    Flowchart,
    State,
    Class,
    Er,
    Gantt,
    Pie,
    Mindmap,
    Timeline,
    Journey,
    Quadrant,
    Gitgraph,
    Requirement,
    C4,
    Sankey,
    Xychart,
    Block,
    Packet,
    Architecture,
    Kanban,
    Radar,
}

/// The keyword each type is declared with on a sheet's first line, as Mermaid reads it.
const DECLARATIONS: [(&str, DiagramKind); 32] = [
    ("sequenceDiagram", DiagramKind::Sequence),
    ("flowchart", DiagramKind::Flowchart),
    ("graph", DiagramKind::Flowchart),
    ("stateDiagram", DiagramKind::State),
    ("stateDiagram-v2", DiagramKind::State),
    ("classDiagram", DiagramKind::Class),
    ("classDiagram-v2", DiagramKind::Class),
    ("erDiagram", DiagramKind::Er),
    ("gantt", DiagramKind::Gantt),
    ("pie", DiagramKind::Pie),
    ("mindmap", DiagramKind::Mindmap),
    ("timeline", DiagramKind::Timeline),
    ("journey", DiagramKind::Journey),
    ("quadrantChart", DiagramKind::Quadrant),
    ("gitGraph", DiagramKind::Gitgraph),
    ("requirementDiagram", DiagramKind::Requirement),
    ("C4Context", DiagramKind::C4),
    ("C4Container", DiagramKind::C4),
    ("C4Component", DiagramKind::C4),
    ("C4Dynamic", DiagramKind::C4),
    ("C4Deployment", DiagramKind::C4),
    ("sankey", DiagramKind::Sankey),
    ("sankey-beta", DiagramKind::Sankey),
    ("xychart", DiagramKind::Xychart),
    ("xychart-beta", DiagramKind::Xychart),
    ("block", DiagramKind::Block),
    ("block-beta", DiagramKind::Block),
    ("packet", DiagramKind::Packet),
    ("packet-beta", DiagramKind::Packet),
    ("architecture-beta", DiagramKind::Architecture),
    ("kanban", DiagramKind::Kanban),
    ("radar-beta", DiagramKind::Radar),
];

impl DiagramKind {
    pub const ALL: [DiagramKind; 21] = [
        DiagramKind::Sequence,
        DiagramKind::Flowchart,
        DiagramKind::State,
        DiagramKind::Class,
        DiagramKind::Er,
        DiagramKind::Gantt,
        DiagramKind::Pie,
        DiagramKind::Mindmap,
        DiagramKind::Timeline,
        DiagramKind::Journey,
        DiagramKind::Quadrant,
        DiagramKind::Gitgraph,
        DiagramKind::Requirement,
        DiagramKind::C4,
        DiagramKind::Sankey,
        DiagramKind::Xychart,
        DiagramKind::Block,
        DiagramKind::Packet,
        DiagramKind::Architecture,
        DiagramKind::Kanban,
        DiagramKind::Radar,
    ];

    /// The type a declaration keyword names: `flowchart` and `graph` are both a flowchart.
    pub fn of_keyword(keyword: &str) -> Option<Self> {
        DECLARATIONS.iter().find(|(declared, _)| *declared == keyword).map(|(_, kind)| *kind)
    }

    /// The stored and spoken name: `sequence`, `er`, `c4`.
    pub fn name(self) -> &'static str {
        match self {
            DiagramKind::Sequence => "sequence",
            DiagramKind::Flowchart => "flowchart",
            DiagramKind::State => "state",
            DiagramKind::Class => "class",
            DiagramKind::Er => "er",
            DiagramKind::Gantt => "gantt",
            DiagramKind::Pie => "pie",
            DiagramKind::Mindmap => "mindmap",
            DiagramKind::Timeline => "timeline",
            DiagramKind::Journey => "journey",
            DiagramKind::Quadrant => "quadrant",
            DiagramKind::Gitgraph => "gitgraph",
            DiagramKind::Requirement => "requirement",
            DiagramKind::C4 => "c4",
            DiagramKind::Sankey => "sankey",
            DiagramKind::Xychart => "xychart",
            DiagramKind::Block => "block",
            DiagramKind::Packet => "packet",
            DiagramKind::Architecture => "architecture",
            DiagramKind::Kanban => "kanban",
            DiagramKind::Radar => "radar",
        }
    }

    pub fn of_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.name() == name)
    }

    /// Whether the console draws this type itself, so its elements can open the code.
    pub fn links_elements(self) -> bool {
        matches!(self, DiagramKind::Sequence | DiagramKind::Flowchart | DiagramKind::State | DiagramKind::Class | DiagramKind::Er)
    }
}

impl std::fmt::Display for DiagramKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
#[path = "../../tests/unit/document/diagram_kind_tests.rs"]
mod tests;
