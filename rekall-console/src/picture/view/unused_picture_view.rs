use rekall_core::picture::PicturePath;

/// A picture no text shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnusedPictureView {
    pub picture: PicturePath,
    pub size_bytes: u64,
}
