/// Why a document could not be written. The message is shown to the person, so it says what failed.
#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("the Word file could not be written: {0}")]
    Docx(String),
    #[error("the PDF could not be written: {0}")]
    Pdf(String),
    #[error("an image could not be read: {0}")]
    Image(String),
}
