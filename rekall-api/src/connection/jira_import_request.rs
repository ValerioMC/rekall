use serde::Deserialize;

/// The keys of the Jira issues to import as the project's tasks.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct JiraImportRequest {
    pub keys: Vec<String>,
}
