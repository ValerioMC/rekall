use serde::Deserialize;

/// How an uploaded picture is asked for: its side, and the version that makes the URL immutable.
#[derive(Debug, Deserialize)]
pub struct ImageSizeQuery {
    /// Side in device pixels; the whole master when left out.
    pub size: Option<u32>,
    /// The image's version: present means the URL names one image for good, so it is cached for good.
    pub v: Option<String>,
}
