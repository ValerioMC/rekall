use super::{InlineRun, ListItem, PictureBlock};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Heading { level: u8, inlines: Vec<InlineRun> },
    Paragraph(Vec<InlineRun>),
    List { ordered: bool, items: Vec<ListItem> },
    CodeBlock { language: Option<String>, text: String },
    Table { header: Vec<Vec<InlineRun>>, rows: Vec<Vec<Vec<InlineRun>>> },
    Quote(Vec<Block>),
    Rule,
    Diagram { png: Vec<u8> },
    DiagramFailed,
    Picture(PictureBlock),
    PictureMissing { description: String },
}
