//! Just enough of Mermaid's text to hold a sheet's links to it: which type it declares, which
//! ids it mentions and how many messages a sequence sends. The console parses it in full.

mod element_key;
mod mermaid_text;

pub use element_key::ElementKey;
pub use mermaid_text::MermaidText;
