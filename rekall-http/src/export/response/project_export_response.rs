use rekall_console::export::ProjectExportView;
use rekall_kernel::{Id, Instant};
use rekall_store::{ExportFailureKind, ExportFormat, ExportStatus};
use serde::Serialize;

/// An export with the project and company a page groups it by.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectExportResponse {
    pub id: Id,
    pub project_id: Id,
    pub project_label: String,
    pub project_title: String,
    pub company_id: Id,
    pub company_name: String,
    pub format: ExportFormat,
    pub status: ExportStatus,
    pub summary: String,
    pub file_name: String,
    pub size_bytes: i64,
    pub diagrams_missing: i32,
    pub failure_kind: Option<ExportFailureKind>,
    pub failure_message: Option<String>,
    pub created_at: Instant,
}

impl From<ProjectExportView> for ProjectExportResponse {
    fn from(view: ProjectExportView) -> Self {
        Self {
            id: view.id,
            project_id: view.project_id,
            project_label: view.project_label,
            project_title: view.project_title,
            company_id: view.company_id,
            company_name: view.company_name,
            format: view.format,
            status: view.status,
            summary: view.summary,
            file_name: view.file_name,
            size_bytes: view.size_bytes,
            diagrams_missing: view.diagrams_missing,
            failure_kind: view.failure_kind,
            failure_message: view.failure_message,
            created_at: view.created_at,
        }
    }
}
