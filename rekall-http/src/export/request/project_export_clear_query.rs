use rekall_console::export::ProjectExportClearCommand;
use rekall_store::ExportStatus;
use serde::Deserialize;

use crate::shared::error::{ApiError, ApiResult};
use crate::shared::extract::optional_uuid;

/// Which exports `DELETE /api/exports` clears: a company's, a project's, those with a status
/// (`done` or `failed`), all that are given together.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ProjectExportClearQuery {
    pub company_id: Option<String>,
    pub project_id: Option<String>,
    pub status: Option<String>,
}

impl TryFrom<ProjectExportClearQuery> for ProjectExportClearCommand {
    type Error = ApiError;

    fn try_from(query: ProjectExportClearQuery) -> ApiResult<Self> {
        Ok(Self {
            company_id: optional_uuid("companyId", query.company_id.as_deref())?,
            project_id: optional_uuid("projectId", query.project_id.as_deref())?,
            status: query.status.as_deref().filter(|status| !status.is_empty()).map(status_of).transpose()?,
        })
    }
}

fn status_of(raw: &str) -> ApiResult<ExportStatus> {
    ExportStatus::value_of(&raw.trim().to_uppercase()).ok_or_else(|| ApiError::Validation(vec![("status".into(), "must be done or failed".into())]))
}
