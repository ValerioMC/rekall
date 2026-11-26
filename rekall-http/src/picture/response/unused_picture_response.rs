use rekall_console::picture::UnusedPictureView;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnusedPictureResponse {
    pub url: String,
    /// Under the images folder, such as `acme/web/web-12`.
    pub folder: String,
    pub size_bytes: u64,
}

impl From<UnusedPictureView> for UnusedPictureResponse {
    fn from(view: UnusedPictureView) -> Self {
        Self { url: view.picture.address(), folder: view.picture.folder().to_string(), size_bytes: view.size_bytes }
    }
}
