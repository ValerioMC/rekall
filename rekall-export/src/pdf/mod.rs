//! The PDF writer: the blocks are laid out in points on A4 pages, then written with printpdf
//! using subsets of the embedded Fira fonts.

mod base_style;
mod code_layout;
mod figure_layout;
mod flow;
mod flow_item;
mod flow_line;
mod flow_piece;
mod line_painter;
mod pdf_blocks;
mod pdf_emitter;
mod pdf_face;
mod pdf_faces;
mod pdf_font_kind;
mod pdf_metrics;
mod pdf_writer;
mod placed;
mod sheet;
mod table_layout;
mod text_style;

pub use pdf_writer::PdfWriter;
