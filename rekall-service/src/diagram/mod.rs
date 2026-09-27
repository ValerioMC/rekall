//! Diagrams: Semantic Graphs stored against a project. `DiagramService` reads them, writes one
//! whole, writes one a session generated for a task after holding its spans against the project
//! folder, and traces a line of code back to the elements it implements. Deleting one is the
//! console's, in rekall-api.

mod diagram_draft;
mod diagram_service;
mod diagram_stream_event;
mod diagram_summary_view;
mod diagram_trace_view;
mod diagram_view;
mod generated_diagram;
mod generated_diagram_view;
mod project_folder;
mod source_audit;

pub use diagram_draft::DiagramDraft;
pub use diagram_service::DiagramService;
pub use diagram_stream_event::DiagramStreamEvent;
pub use diagram_summary_view::DiagramSummaryView;
pub use diagram_trace_view::DiagramTraceView;
pub use diagram_view::DiagramView;
pub use generated_diagram::GeneratedDiagram;
pub use generated_diagram_view::GeneratedDiagramView;
pub use project_folder::ProjectFolder;
pub use source_audit::SourceAudit;
