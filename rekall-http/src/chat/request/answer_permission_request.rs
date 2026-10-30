use serde::Deserialize;

use crate::shared::extract::present;
use crate::shared::ApiResult;

/// The person's answer to a permission prompt.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AnswerPermissionRequest {
    pub allow: Option<bool>,
}

impl AnswerPermissionRequest {
    pub fn allowed(&self) -> ApiResult<bool> {
        present("allow", self.allow)
    }
}
