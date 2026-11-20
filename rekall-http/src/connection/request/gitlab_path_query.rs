use serde::Deserialize;

/// The path a person has typed or pasted for a GitLab binding.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GitlabPathQuery {
    pub path: Option<String>,
}
