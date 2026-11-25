/// A finished export as the browser downloads it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportedFile {
    pub file_name: String,
    pub media_type: &'static str,
    pub bytes: Vec<u8>,
}
