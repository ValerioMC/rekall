/// A stretch of text that shares one style. A hard line break is a `\n` in the text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineRun {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub code: bool,
    pub link: Option<String>,
}

impl InlineRun {
    pub fn plain(text: impl Into<String>) -> Self {
        Self { text: text.into(), bold: false, italic: false, code: false, link: None }
    }
}
