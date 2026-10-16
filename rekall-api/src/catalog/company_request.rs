use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CompanyRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}
