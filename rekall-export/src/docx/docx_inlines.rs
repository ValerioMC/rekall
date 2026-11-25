use docx_rs::{BreakType, Hyperlink, HyperlinkType, Paragraph, Run, RunFonts};

use crate::markdown::InlineRun;
use crate::style::DocumentStyle;

use super::docx_measure::DocxMeasure;

pub(super) struct DocxInlines;

impl DocxInlines {
    pub(super) fn fill(mut paragraph: Paragraph, inlines: &[InlineRun], bold: bool) -> Paragraph {
        for inline in inlines {
            let run = Self::run(inline, bold);
            paragraph = match &inline.link {
                Some(target) => paragraph.add_hyperlink(Hyperlink::new(target, HyperlinkType::External).add_run(run)),
                None => paragraph.add_run(run),
            };
        }
        paragraph
    }

    pub(super) fn code_fonts() -> RunFonts {
        let font = DocumentStyle::DOCX_CODE_FONT;
        RunFonts::new().ascii(font).hi_ansi(font).east_asia(font).cs(font)
    }

    pub(super) fn text_run(text: &str) -> Run {
        let mut lines = text.split('\n');
        let mut run = Run::new().add_text(lines.next().unwrap_or_default().replace('\t', "    "));
        for line in lines {
            run = run.add_break(BreakType::TextWrapping).add_text(line.replace('\t', "    "));
        }
        run
    }

    fn run(inline: &InlineRun, bold: bool) -> Run {
        let mut run = Self::text_run(&inline.text);
        if bold || inline.bold {
            run = run.bold();
        }
        if inline.italic {
            run = run.italic();
        }
        if inline.code {
            run = run.fonts(Self::code_fonts()).size(DocxMeasure::half_points(DocumentStyle::CODE_POINTS));
        }
        if inline.link.is_some() {
            run = run.color(DocumentStyle::LINK.hex()).underline("single");
        }
        run
    }
}
