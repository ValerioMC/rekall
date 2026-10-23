use rekall_common::{Id, Instant};
use rekall_model::{project_dependency, Ecosystem};
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDependencyView {
    pub id: Id,
    pub project_id: Id,
    pub ecosystem: Ecosystem,
    pub name: String,
    pub version: String,
    pub manifest: String,
    pub created_at: Instant,
}

impl ProjectDependencyView {
    pub fn of(row: &project_dependency::Model) -> Self {
        Self {
            id: row.id,
            project_id: row.project_id,
            ecosystem: row.ecosystem,
            name: row.name.clone(),
            version: row.version.clone(),
            manifest: row.manifest.clone(),
            created_at: row.created_at,
        }
    }
}
