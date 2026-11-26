use pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd};

use crate::document::{DiagramImage, ExportPicture};
use crate::style::DocumentStyle;

use super::mermaid_fence::is_mermaid;
use super::picture_address::PictureAddress;
use super::{Block, InlineRun, ListItem, PictureBlock};

enum FrameKind {
    Root,
    Quote,
    List { ordered: bool, items: Vec<ListItem> },
    Item { checked: Option<bool> },
}

struct Frame {
    kind: FrameKind,
    blocks: Vec<Block>,
}

struct CodeBuffer {
    language: Option<String>,
    mermaid: bool,
    text: String,
}

struct ImageBuffer {
    address: String,
    description: String,
}

#[derive(Default)]
struct TableBuilder {
    header: Vec<Vec<InlineRun>>,
    rows: Vec<Vec<Vec<InlineRun>>>,
    row: Vec<Vec<InlineRun>>,
}

#[derive(Default)]
struct Emphasis {
    bold: bool,
    italic: bool,
    link: Option<String>,
}

/// Turns the events of one markdown part into blocks. Text that sits directly in a tight list
/// item is gathered into a paragraph when the item ends or a nested block starts.
pub(super) struct BlockBuilder<'a> {
    diagrams: &'a [DiagramImage],
    pictures: &'a [ExportPicture],
    frames: Vec<Frame>,
    runs: Vec<InlineRun>,
    emphasis: Emphasis,
    heading_level: Option<u8>,
    code: Option<CodeBuffer>,
    table: Option<TableBuilder>,
    image: Option<ImageBuffer>,
}

impl<'a> BlockBuilder<'a> {
    pub(super) fn new(part_heading: &str, diagrams: &'a [DiagramImage], pictures: &'a [ExportPicture]) -> Self {
        let heading = Block::Heading { level: 1, inlines: vec![InlineRun::plain(part_heading)] };
        Self {
            diagrams,
            pictures,
            frames: vec![Frame { kind: FrameKind::Root, blocks: vec![heading] }],
            runs: Vec::new(),
            emphasis: Emphasis::default(),
            heading_level: None,
            code: None,
            table: None,
            image: None,
        }
    }

    pub(super) fn finish(mut self) -> Vec<Block> {
        self.flush_paragraph();
        self.frames.into_iter().next().map(|root| root.blocks).unwrap_or_default()
    }

    pub(super) fn accept(&mut self, event: Event<'_>) {
        match event {
            Event::Start(tag) => self.start(tag),
            Event::End(tag) => self.end(tag),
            Event::Text(text) => self.text(&text),
            Event::Code(text) => self.push_run(&text, true),
            Event::SoftBreak => self.push_run(" ", false),
            Event::HardBreak => self.push_run("\n", false),
            Event::Rule => self.push_block(Block::Rule),
            Event::TaskListMarker(checked) => self.mark_item(checked),
            _ => {}
        }
    }

    fn start(&mut self, tag: Tag<'_>) {
        match tag {
            Tag::Heading { level, .. } => self.heading_level = Some(Self::shifted(level as u8)),
            Tag::BlockQuote(_) => self.open(FrameKind::Quote),
            Tag::List(first) => {
                self.flush_paragraph();
                self.open(FrameKind::List { ordered: first.is_some(), items: Vec::new() });
            }
            Tag::Item => self.open(FrameKind::Item { checked: None }),
            Tag::CodeBlock(kind) => self.code = Some(Self::code_buffer(&kind)),
            Tag::Table(_) => self.table = Some(TableBuilder::default()),
            Tag::Strong => self.emphasis.bold = true,
            Tag::Emphasis => self.emphasis.italic = true,
            Tag::Link { dest_url, .. } => self.emphasis.link = Some(dest_url.to_string()),
            Tag::Image { dest_url, .. } => self.image = Some(ImageBuffer { address: dest_url.to_string(), description: String::new() }),
            _ => {}
        }
    }

    fn end(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Paragraph => self.flush_paragraph(),
            TagEnd::Heading(_) => self.close_heading(),
            TagEnd::BlockQuote(_) => self.close_quote(),
            TagEnd::List(_) => self.close_list(),
            TagEnd::Item => self.close_item(),
            TagEnd::CodeBlock => self.close_code(),
            TagEnd::TableCell => self.close_cell(),
            TagEnd::TableHead => self.close_header(),
            TagEnd::TableRow => self.close_row(),
            TagEnd::Table => self.close_table(),
            TagEnd::Strong => self.emphasis.bold = false,
            TagEnd::Emphasis => self.emphasis.italic = false,
            TagEnd::Link => self.emphasis.link = None,
            TagEnd::Image => self.close_image(),
            _ => {}
        }
    }

    fn shifted(level: u8) -> u8 {
        (level + 1).min(DocumentStyle::MAX_HEADING_LEVEL)
    }

    fn code_buffer(kind: &CodeBlockKind<'_>) -> CodeBuffer {
        let language = match kind {
            CodeBlockKind::Fenced(info) => info.split_whitespace().next().map(str::to_string),
            CodeBlockKind::Indented => None,
        };
        CodeBuffer { language, mermaid: is_mermaid(kind), text: String::new() }
    }

    fn text(&mut self, text: &str) {
        match self.code.as_mut() {
            Some(code) => code.text.push_str(text),
            None => self.push_run(text, false),
        }
    }

    /// Text inside a picture is its description, not part of the line.
    fn push_run(&mut self, text: &str, code: bool) {
        if let Some(image) = self.image.as_mut() {
            image.description.push_str(text);
            return;
        }
        self.runs.push(InlineRun {
            text: text.to_string(),
            bold: self.emphasis.bold,
            italic: self.emphasis.italic,
            code,
            link: self.emphasis.link.clone(),
        });
    }

    fn open(&mut self, kind: FrameKind) {
        self.frames.push(Frame { kind, blocks: Vec::new() });
    }

    fn push_block(&mut self, block: Block) {
        if let Some(frame) = self.frames.last_mut() {
            frame.blocks.push(block);
        }
    }

    fn flush_paragraph(&mut self) {
        let runs = std::mem::take(&mut self.runs);
        if runs.iter().any(|run| !run.text.trim().is_empty()) {
            self.push_block(Block::Paragraph(runs));
        }
    }

    /// A picture splits the paragraph it sits in. A heading and a table cell hold text alone.
    fn close_image(&mut self) {
        let Some(image) = self.image.take() else { return };
        let address = PictureAddress::read(&image.address);
        if address.is_web() {
            return self.push_web_picture(&image.description, address.path);
        }
        if self.heading_level.is_some() || self.table.is_some() {
            return self.push_run(&image.description, false);
        }
        let block = self.picture_block(&address, image.description);
        self.flush_paragraph();
        self.push_block(block);
    }

    fn picture_block(&self, address: &PictureAddress<'_>, description: String) -> Block {
        match self.pictures.iter().find(|picture| picture.address == address.path) {
            Some(picture) => Block::Picture(PictureBlock {
                png: picture.png.clone(),
                natural_width: picture.natural_width,
                alignment: address.alignment,
            }),
            None => Block::PictureMissing { description },
        }
    }

    fn push_web_picture(&mut self, description: &str, address: &str) {
        if !description.is_empty() {
            self.push_run(&format!("{description} "), false);
        }
        let outer_link = self.emphasis.link.replace(address.to_string());
        self.push_run(address, false);
        self.emphasis.link = outer_link;
    }

    fn close_heading(&mut self) {
        let level = self.heading_level.take().unwrap_or(DocumentStyle::MAX_HEADING_LEVEL);
        let inlines = std::mem::take(&mut self.runs);
        self.push_block(Block::Heading { level, inlines });
    }

    fn close_quote(&mut self) {
        self.flush_paragraph();
        if let Some(Frame { kind: FrameKind::Quote, blocks }) = self.frames.pop() {
            self.push_block(Block::Quote(blocks));
        }
    }

    fn close_list(&mut self) {
        if let Some(Frame { kind: FrameKind::List { ordered, items }, .. }) = self.frames.pop() {
            self.push_block(Block::List { ordered, items });
        }
    }

    fn close_item(&mut self) {
        self.flush_paragraph();
        let Some(Frame { kind: FrameKind::Item { checked }, blocks }) = self.frames.pop() else { return };
        if let Some(Frame { kind: FrameKind::List { items, .. }, .. }) = self.frames.last_mut() {
            items.push(ListItem { checked, blocks });
        }
    }

    fn mark_item(&mut self, checked: bool) {
        if let Some(Frame { kind: FrameKind::Item { checked: marker }, .. }) = self.frames.last_mut() {
            *marker = Some(checked);
        }
    }

    fn close_code(&mut self) {
        let Some(code) = self.code.take() else { return };
        self.push_block(self.code_block(code));
    }

    fn code_block(&self, code: CodeBuffer) -> Block {
        if !code.mermaid {
            return Block::CodeBlock { language: code.language, text: code.text.trim_end_matches('\n').to_string() };
        }
        let source = code.text.trim();
        match self.diagrams.iter().find(|image| image.source.trim() == source) {
            Some(image) => Block::Diagram { png: image.png.clone() },
            None => Block::DiagramFailed,
        }
    }

    fn close_cell(&mut self) {
        let cell = std::mem::take(&mut self.runs);
        if let Some(table) = self.table.as_mut() {
            table.row.push(cell);
        }
    }

    fn close_header(&mut self) {
        if let Some(table) = self.table.as_mut() {
            table.header = std::mem::take(&mut table.row);
        }
    }

    fn close_row(&mut self) {
        if let Some(table) = self.table.as_mut() {
            let row = std::mem::take(&mut table.row);
            table.rows.push(row);
        }
    }

    fn close_table(&mut self) {
        if let Some(table) = self.table.take() {
            self.push_block(Block::Table { header: table.header, rows: table.rows });
        }
    }
}
