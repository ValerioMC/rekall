use crate::diagram::DiagramView;

/// What writing a generated diagram left behind. `sources_checked` is false when the project has
/// no folder, so its spans could not be held against the code.
#[derive(Clone, Debug, PartialEq)]
pub struct GeneratedDiagramView {
    pub diagram: DiagramView,
    /// The project, or the project and task, it was stored on.
    pub anchors: String,
    pub replaced: bool,
    pub sources_checked: bool,
}
