use std::ops::Range;

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

use crate::picture::PicturePath;

/// One picture of Rekall's folder that a markdown text shows: where its markup sits in the text,
/// the file, and its description.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PictureReference {
    pub span: Range<usize>,
    pub picture: PicturePath,
    pub description: String,
}

impl PictureReference {
    /// In the order of the text. Code shows no picture, and an alignment ending is no part of a file.
    pub fn all_in(markdown: &str) -> Vec<Self> {
        let options = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
        let mut found = Vec::new();
        let mut open: Option<Self> = None;
        let mut depth = 0_usize;
        for (event, span) in Parser::new_ext(markdown, options).into_offset_iter() {
            match event {
                Event::Start(Tag::Image { dest_url, .. }) => {
                    if depth == 0 {
                        open = Self::of(&dest_url, span);
                    }
                    depth += 1;
                }
                Event::End(TagEnd::Image) => {
                    depth -= 1;
                    if depth == 0 {
                        found.extend(open.take());
                    }
                }
                Event::Text(text) | Event::Code(text) => {
                    if let Some(reference) = open.as_mut() {
                        reference.description.push_str(&text);
                    }
                }
                Event::SoftBreak | Event::HardBreak => {
                    if let Some(reference) = open.as_mut() {
                        reference.description.push(' ');
                    }
                }
                _ => {}
            }
        }
        found
    }

    /// The markdown with the markup of each of its pictures replaced by what `written` says for it.
    pub fn replaced_in(markdown: &str, written: impl Fn(&Self) -> String) -> String {
        let mut replaced = String::with_capacity(markdown.len());
        let mut kept_from = 0;
        for reference in Self::all_in(markdown) {
            replaced.push_str(&markdown[kept_from..reference.span.start]);
            replaced.push_str(&written(&reference));
            kept_from = reference.span.end;
        }
        replaced.push_str(&markdown[kept_from..]);
        replaced
    }

    fn of(destination: &str, span: Range<usize>) -> Option<Self> {
        let address = destination.split('#').next().unwrap_or_default();
        let picture = PicturePath::of_address(address).ok()?;
        Some(Self { span, picture, description: String::new() })
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/picture/rule/picture_reference_tests.rs"]
mod tests;
