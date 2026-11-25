/// One page, component or blueprint: its heading and its markdown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportPart {
    pub heading: String,
    pub markdown: String,
}
