use rekall_core::component::ProjectComponentView;

/// What scanning a project's folder did: the components it added, and how many of the
/// applications it found already had one.
#[derive(Clone, Debug)]
pub struct ComponentScanView {
    pub folder: String,
    pub added: Vec<ProjectComponentView>,
    pub already_known: usize,
}
