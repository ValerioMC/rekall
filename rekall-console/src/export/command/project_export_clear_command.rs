use rekall_kernel::Id;
use rekall_store::ExportStatus;

/// Which exports to delete: those of a company, of a project, with a status, all that apply together.
#[derive(Clone, Debug, Default)]
pub struct ProjectExportClearCommand {
    pub company_id: Option<Id>,
    pub project_id: Option<Id>,
    pub status: Option<ExportStatus>,
}
