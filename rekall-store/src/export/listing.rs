use rekall_kernel::Id;

use crate::project_export;

/// An export with the names of the project and company a page groups it by.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectExportListing {
    pub export: project_export::Model,
    pub project_label: String,
    pub project_title: String,
    pub company_id: Id,
    pub company_name: String,
}
