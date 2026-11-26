/// A picture handed to the writers, found again by its markdown address without the `#` ending.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportPicture {
    pub address: String,
    pub png: Vec<u8>,
    /// In pixels of 96 to the inch. It is never drawn wider; a PNG with more pixels is sharper.
    pub natural_width: u32,
}
