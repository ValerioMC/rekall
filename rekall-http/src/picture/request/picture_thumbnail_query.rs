use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PictureThumbnailQuery {
    pub path: Option<String>,
}
