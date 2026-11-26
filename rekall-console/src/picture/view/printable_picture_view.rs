/// A kept picture as a document prints it: a PNG, and the width in pixels it has when not shrunk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrintablePictureView {
    pub png: Vec<u8>,
    pub natural_width: u32,
}
