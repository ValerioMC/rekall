//! Diagrams: Mermaid sheets stored against a project, with their elements tied to code.
//! `DiagramService` reads them, writes one whole, and writes one a session generated for a
//! project or a task after holding its spans against the project folder. Deleting one is the
//! console's, in rekall-console.

mod event;
mod model;
mod service;
mod view;

pub use model::diagram_draft::DiagramDraft;
pub use service::diagram_service::DiagramService;
pub use view::diagram_sheet_view::DiagramSheetView;
pub use event::diagram_stream_event::DiagramStreamEvent;
pub use view::diagram_summary_view::DiagramSummaryView;
pub use model::diagram_target::DiagramTarget;
pub use view::diagram_view::DiagramView;
pub use model::generated_diagram::GeneratedDiagram;
pub use view::generated_diagram_view::GeneratedDiagramView;
pub use model::project_folder::ProjectFolder;
pub use model::source_audit::SourceAudit;
