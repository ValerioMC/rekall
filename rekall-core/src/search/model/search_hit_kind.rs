use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SearchHitKind {
    Description,
    Step,
    Wrapup,
    Note,
}
