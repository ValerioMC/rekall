use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

use crate::document::{DiagramImage, ExportPart, ExportPicture};

use super::block_builder::BlockBuilder;
use super::mermaid_fence::is_mermaid;
use super::picture_address::PictureAddress;
use super::Block;

pub struct MarkdownBlocks;

impl MarkdownBlocks {
    /// The part's heading at level 1, then its markdown with its own headings one level down.
    pub fn read(part: &ExportPart, diagrams: &[DiagramImage], pictures: &[ExportPicture]) -> Vec<Block> {
        let mut builder = BlockBuilder::new(&part.heading, diagrams, pictures);
        for event in Parser::new_ext(&part.markdown, Self::options()) {
            builder.accept(event);
        }
        builder.finish()
    }

    /// The trimmed source of every mermaid fence, in document order.
    pub fn mermaid_sources(markdown: &str) -> Vec<String> {
        let mut sources = Vec::new();
        let mut open: Option<String> = None;
        for event in Parser::new_ext(markdown, Self::options()) {
            match event {
                Event::Start(Tag::CodeBlock(kind)) if is_mermaid(&kind) => open = Some(String::new()),
                Event::Text(text) => {
                    if let Some(source) = open.as_mut() {
                        source.push_str(&text);
                    }
                }
                Event::End(TagEnd::CodeBlock) => sources.extend(open.take().map(|source| source.trim().to_string())),
                _ => {}
            }
        }
        sources
    }

    /// The address, without its alignment ending, of every picture not on the web, in document order.
    pub(crate) fn local_picture_addresses(markdown: &str) -> Vec<String> {
        Parser::new_ext(markdown, Self::options())
            .filter_map(|event| match event {
                Event::Start(Tag::Image { dest_url, .. }) => {
                    let address = PictureAddress::read(&dest_url);
                    (!address.is_web()).then(|| address.path.to_string())
                }
                _ => None,
            })
            .collect()
    }

    fn options() -> Options {
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS
    }
}

#[cfg(test)]
#[path = "../../tests/unit/markdown/markdown_blocks_tests.rs"]
mod tests;
