use rekall_console::component::ComponentScanView;
use rekall_core::component::ProjectComponentView;
use serde::Serialize;

/// What scanning a project's folder did: the components it added, and how many of the
/// repositories it found already had one.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentScanResponse {
    pub folder: String,
    pub added: Vec<ProjectComponentView>,
    pub already_known: usize,
}

impl From<ComponentScanView> for ComponentScanResponse {
    fn from(view: ComponentScanView) -> Self {
        Self { folder: view.folder, added: view.added, already_known: view.already_known }
    }
}
