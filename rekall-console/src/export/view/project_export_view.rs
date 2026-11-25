use rekall_kernel::{Id, Instant};
use rekall_store::export::ProjectExportListing;
use rekall_store::{ExportFailureKind, ExportFormat, ExportStatus};

/// An export with the project and company a page groups it by.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectExportView {
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

impl From<ProjectExportListing> for ProjectExportView {
    fn from(listing: ProjectExportListing) -> Self {
        let export = listing.export;
        Self {
            id: export.id,
            project_id: export.project_id,
            project_label: listing.project_label,
            project_title: listing.project_title,
            company_id: listing.company_id,
            company_name: listing.company_name,
            format: export.format,
            status: export.status,
            summary: export.summary,
            file_name: export.file_name,
            size_bytes: export.size_bytes,
            diagrams_missing: export.diagrams_missing,
            failure_kind: export.failure_kind,
            failure_message: export.failure_message,
            created_at: export.created_at,
        }
    }
}
