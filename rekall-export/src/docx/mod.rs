//! The Word writer: one block becomes one Word construct, in the fonts of `DocumentStyle`.

mod docx_blocks;
mod docx_element;
mod docx_figures;
mod docx_inlines;
mod docx_measure;
mod docx_styles;
mod docx_writer;

pub use docx_writer::DocxWriter;
