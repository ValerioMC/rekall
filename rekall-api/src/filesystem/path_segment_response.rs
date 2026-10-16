use serde::Serialize;

/// One crumb of a listing's breadcrumb, from the filesystem root down to the folder itself.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PathSegmentResponse {
    pub name: String,
    pub path: String,
}
