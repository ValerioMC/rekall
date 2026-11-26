use rekall_core::picture::PictureFormat;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PictureFileView {
    pub bytes: Vec<u8>,
    pub format: PictureFormat,
}
