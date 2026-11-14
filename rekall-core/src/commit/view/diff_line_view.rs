use serde::Serialize;

use super::DiffLineKind;

/// One line of a hunk. A removed line has no new number and an added one has no old number.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffLineView {
    pub kind: DiffLineKind,
    pub old_number: Option<u32>,
    pub new_number: Option<u32>,
    pub text: String,
}
