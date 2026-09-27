//! Diagrams: Semantic Graphs stored against a project. `DiagramService` reads them, writes one
//! whole (the path a session will use too), and traces a line of code back to the elements it
//! implements. Deleting one is the console's, in rekall-api.

mod diagram_draft;
mod diagram_service;
mod diagram_stream_event;
mod diagram_summary_view;
mod diagram_trace_view;
mod diagram_view;

pub use diagram_draft::DiagramDraft;
pub use diagram_service::DiagramService;
pub use diagram_stream_event::DiagramStreamEvent;
pub use diagram_summary_view::DiagramSummaryView;
pub use diagram_trace_view::DiagramTraceView;
pub use diagram_view::DiagramView;
