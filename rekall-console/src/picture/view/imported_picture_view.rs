/// A picture now kept beside the database, by the address the markdown shows it with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportedPictureView {
    pub url: String,
}
