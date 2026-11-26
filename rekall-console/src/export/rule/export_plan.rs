use rekall_export::{DiagramImage, ExportDocument};
use rekall_store::{project, ExportFormat};

use super::export_file_name;
use super::export_selection::ExportSelection;
use crate::export::DiagramImageCommand;

/// Everything an export needs once the items are read: the document to write, still without its
/// pictures, what it is called and what the record of it says.
#[derive(Clone, Debug)]
pub struct ExportPlan {
    pub project: project::Model,
    pub format: ExportFormat,
    pub document: ExportDocument,
    pub summary: String,
    pub file_name: String,
    pub diagrams_missing: i32,
}

impl ExportPlan {
    pub fn of(project: project::Model, format: ExportFormat, selection: &ExportSelection, diagrams: &[DiagramImageCommand]) -> Self {
        let document = ExportDocument {
            title: project.title.clone(),
            parts: selection.parts(),
            diagrams: diagrams.iter().map(|image| DiagramImage { source: image.source.clone(), png: image.png.clone() }).collect(),
            pictures: Vec::new(),
        };
        Self {
            summary: selection.summary(),
            file_name: export_file_name::of_export(&project.label, selection.only_title().as_deref(), format),
            diagrams_missing: i32::try_from(document.diagrams_missing()).unwrap_or(i32::MAX),
            project,
            format,
            document,
        }
    }
}
