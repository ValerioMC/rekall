//! A project's pages, components and blueprint as one PDF or Word file.
//!
//! It knows nothing of Rekall's database or console: it takes an `ExportDocument` of markdown parts,
//! the images of their diagrams and their pictures, and returns bytes. `markdown` reads a part into
//! the blocks both writers draw, `style` holds the look they share.

pub mod docx;
pub mod document;
pub mod image;
pub mod markdown;
pub mod pdf;
pub mod style;

mod export_error;

pub use docx::DocxWriter;
pub use image::PngSize;
pub use document::{DiagramImage, ExportDocument, ExportPart, ExportPicture};
pub use export_error::ExportError;
pub use pdf::PdfWriter;
pub use markdown::{Block, InlineRun, ListItem, MarkdownBlocks, PictureAlignment, PictureBlock};
pub use style::{DocumentStyle, Rgb};
