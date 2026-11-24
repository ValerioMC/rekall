use rekall_console::section::{ContextPlacementCommand, SectionLayoutCommand};
use serde::Deserialize;

/// Where one context's sections go, by id and in order.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ContextPlacementRequest {
    pub context_id: Option<String>,
    pub section_ids: Option<Vec<String>>,
}

/// A project's whole arrangement: its contexts in the order to be shown, each with its sections.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SectionLayoutRequest {
    pub contexts: Option<Vec<ContextPlacementRequest>>,
}

impl From<SectionLayoutRequest> for SectionLayoutCommand {
    fn from(request: SectionLayoutRequest) -> Self {
        Self {
            contexts: request.contexts.map(|contexts| {
                contexts
                    .into_iter()
                    .map(|placement| ContextPlacementCommand { context_id: placement.context_id, section_ids: placement.section_ids })
                    .collect()
            }),
        }
    }
}
