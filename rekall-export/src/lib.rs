//! A project's pages, components and blueprint as one PDF or Word file.
//!
//! It knows nothing of Rekall's database or console: it takes an `ExportDocument` of markdown parts
//! and the images of their diagrams, and returns bytes. `markdown` reads a part into the blocks both
//! writers draw, `style` holds the look they share.

pub mod document;
pub mod markdown;
pub mod style;

mod export_error;

pub use document::{DiagramImage, ExportDocument, ExportPart};
pub use export_error::ExportError;
pub use markdown::{Block, InlineRun, ListItem, MarkdownBlocks};
pub use style::{DocumentStyle, Rgb};
