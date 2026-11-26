use serde::Deserialize;

use crate::picture::request::picture_owner_request::PictureOwnerRequest;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PictureImportRequest {
    pub source_path: Option<String>,
    pub owner: Option<PictureOwnerRequest>,
}
