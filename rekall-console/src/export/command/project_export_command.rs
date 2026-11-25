use rekall_kernel::Id;
use rekall_store::ExportFormat;

use super::diagram_image_command::DiagramImageCommand;

/// What to export from one project: the items ticked, the file format and the diagram pictures.
#[derive(Clone, Debug)]
pub struct ProjectExportCommand {
    pub format: ExportFormat,
    pub blueprint: bool,
    pub section_ids: Vec<Id>,
    pub component_ids: Vec<Id>,
    pub diagrams: Vec<DiagramImageCommand>,
}
