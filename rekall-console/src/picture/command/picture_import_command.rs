use crate::picture::PictureOwner;

/// A file on this machine to keep for one owner.
#[derive(Clone, Debug)]
pub struct PictureImportCommand {
    pub source_path: String,
    pub owner: PictureOwner,
}
