use serde::Serialize;

use crate::context::ContextSizePart;

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextSize {
    pub characters: usize,
    pub estimated_tokens: usize,
    pub parts: Vec<ContextSizePart>,
}
