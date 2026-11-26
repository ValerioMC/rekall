use crate::picture::PictureOwner;

/// A picture that arrived as bytes, pasted or dropped, to keep for one owner. `name` is the file
/// it came from, if it had one.
#[derive(Clone, Debug)]
pub struct PictureUploadCommand {
    pub name: Option<String>,
    pub bytes: Vec<u8>,
    pub owner: PictureOwner,
}
