use serde::Deserialize;

use crate::picture::request::picture_owner_kind::PictureOwnerKind;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PictureUploadQuery {
    pub owner_kind: Option<PictureOwnerKind>,
    pub owner_id: Option<String>,
    pub name: Option<String>,
}
