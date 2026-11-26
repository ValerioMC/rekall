use serde::Serialize;

/// What a change set does to one of its files.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FileChangeKind {
    Add,
    Delete,
    Update,
}
