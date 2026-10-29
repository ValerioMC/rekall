//! Diagrams: Mermaid sheets stored against a project, with their elements tied to code.
//! `DiagramService` reads them, writes one whole, and writes one a session generated for a
//! project or a task after holding its spans against the project folder. Deleting one is the
//! console's, in rekall-console.

mod diagram_draft;
mod diagram_service;
mod diagram_sheet_view;
mod diagram_stream_event;
mod diagram_summary_view;
mod diagram_target;
mod diagram_view;
mod generated_diagram;
mod generated_diagram_view;
mod project_folder;
mod source_audit;

pub use diagram_draft::DiagramDraft;
pub use diagram_service::DiagramService;
pub use diagram_sheet_view::DiagramSheetView;
pub use diagram_stream_event::DiagramStreamEvent;
pub use diagram_summary_view::DiagramSummaryView;
pub use diagram_target::DiagramTarget;
pub use diagram_view::DiagramView;
pub use generated_diagram::GeneratedDiagram;
pub use generated_diagram_view::GeneratedDiagramView;
pub use project_folder::ProjectFolder;
pub use source_audit::SourceAudit;
