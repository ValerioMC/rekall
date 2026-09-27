use serde::Deserialize;

/// A line of a file in a project's folder, to trace back to diagram elements.
#[derive(Deserialize)]
pub struct DiagramTraceQuery {
    pub file: Option<String>,
    pub line: Option<u32>,
}
