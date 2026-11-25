use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use rekall_console::export::{DiagramImageCommand, ProjectExportCommand};
use rekall_kernel::Id;
use rekall_store::ExportFormat;
use serde::Deserialize;

use super::diagram_image_request::DiagramImageRequest;
use crate::shared::error::{ApiError, ApiResult};
use crate::shared::extract::{present, uuid};

/// What to export from one project: `format` is `pdf` or `docx`, the ids are the pages and
/// components ticked, and each diagram is a source with its PNG as base64.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ProjectExportRequest {
    pub format: Option<String>,
    pub blueprint: bool,
    pub section_ids: Vec<String>,
    pub component_ids: Vec<String>,
    pub diagrams: Vec<DiagramImageRequest>,
}

impl TryFrom<ProjectExportRequest> for ProjectExportCommand {
    type Error = ApiError;

    fn try_from(request: ProjectExportRequest) -> ApiResult<Self> {
        Ok(Self {
            format: format_of(request.format)?,
            blueprint: request.blueprint,
            section_ids: ids_of("sectionIds", &request.section_ids)?,
            component_ids: ids_of("componentIds", &request.component_ids)?,
            diagrams: request.diagrams.into_iter().enumerate().map(|(index, image)| diagram_of(index, image)).collect::<ApiResult<_>>()?,
        })
    }
}

fn format_of(raw: Option<String>) -> ApiResult<ExportFormat> {
    let text = present("format", raw)?;
    ExportFormat::value_of(&text.trim().to_uppercase()).ok_or_else(|| ApiError::Validation(vec![("format".into(), "must be pdf or docx".into())]))
}

fn ids_of(field: &str, raw: &[String]) -> ApiResult<Vec<Id>> {
    raw.iter().map(|id| uuid(field, id)).collect()
}

fn diagram_of(index: usize, image: DiagramImageRequest) -> ApiResult<DiagramImageCommand> {
    let source = present(&format!("diagrams[{index}].source"), image.source)?;
    let encoded = present(&format!("diagrams[{index}].png"), image.png)?;
    let png = STANDARD
        .decode(encoded.trim())
        .map_err(|_| ApiError::Validation(vec![(format!("diagrams[{index}].png"), "must be base64".into())]))?;
    Ok(DiagramImageCommand { source, png })
}
