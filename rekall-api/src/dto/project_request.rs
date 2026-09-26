use rekall_common::Id;
use rekall_model::ProjectStatus;
use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ProjectRequest {
    pub label: Option<String>,
    pub title: Option<String>,
    pub status: Option<ProjectStatus>,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub blueprint_markdown: Option<String>,
    pub repo_folder: Option<String>,
    pub auto_commit: Option<bool>,
    pub company_id: Option<Id>,
}
