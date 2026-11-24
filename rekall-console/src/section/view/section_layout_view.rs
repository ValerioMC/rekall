use rekall_core::section::{ProjectSectionView, SectionContextView};
use serde::Serialize;

/// A project's contexts and sections as arranged, each list in the order shown.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectionLayoutView {
    pub contexts: Vec<SectionContextView>,
    pub sections: Vec<ProjectSectionView>,
}
