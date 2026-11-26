use rekall_kernel::{RekallError, Result};

use crate::picture::PictureFormat;

/// A company, a project and one or two folders under them.
const FOLDER_COUNTS: std::ops::RangeInclusive<usize> = 3..=4;

/// A picture's place under the images folder, such as `acme/web/web-12/login-3f9a1c2b.png`.
/// Only this shape names a file there, so no address reaches outside the folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PicturePath {
    relative: String,
    format: PictureFormat,
}

impl PicturePath {
    pub fn parse(relative: &str) -> Result<Self> {
        let segments: Vec<&str> = relative.split('/').collect();
        let shaped = FOLDER_COUNTS.contains(&(segments.len() - 1)) && segments.iter().all(|segment| is_segment(segment));
        match relative.rsplit_once('.').and_then(|(_, extension)| PictureFormat::of_extension(extension)) {
            Some(format) if shaped => Ok(Self { relative: relative.to_string(), format }),
            _ => Err(RekallError::illegal(format!("'{relative}' is not a picture path."))),
        }
    }

    pub fn relative(&self) -> &str {
        &self.relative
    }

    pub fn format(&self) -> PictureFormat {
        self.format
    }

    /// What the markdown holds to show this picture.
    pub fn address(&self) -> String {
        format!("/api/images/{}", self.relative)
    }
}

fn is_segment(segment: &str) -> bool {
    !matches!(segment, "" | "." | "..")
        && segment.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-'))
}

#[cfg(test)]
#[path = "../../../tests/unit/picture/model/picture_path_tests.rs"]
mod tests;
