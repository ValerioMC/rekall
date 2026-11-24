use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(in crate::chat) enum WireWriteKind {
    Create,
    Update,
}
