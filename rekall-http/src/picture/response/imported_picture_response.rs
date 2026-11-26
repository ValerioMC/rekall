use rekall_console::picture::ImportedPictureView;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ImportedPictureResponse {
    pub url: String,
}

impl From<ImportedPictureView> for ImportedPictureResponse {
    fn from(view: ImportedPictureView) -> Self {
        Self { url: view.url }
    }
}
