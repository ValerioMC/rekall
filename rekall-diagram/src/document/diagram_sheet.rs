use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{DiagramKind, SourceLocation};
use crate::mermaid::MermaidText;

/// One Mermaid diagram and the code its elements are implemented by. A `links` key is an
/// element's Mermaid id (`OrderService`, `validate`), or `#n` for a sequence's n-th message.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DiagramSheet {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub mermaid: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub links: BTreeMap<String, Vec<SourceLocation>>,
}

impl DiagramSheet {
    /// The type the Mermaid's first line declares; none when it declares nothing known.
    pub fn kind(&self) -> Option<DiagramKind> {
        MermaidText::new(&self.mermaid).declared_kind()
    }
}
