//! The description of the record, with the component, connection and task references it names expanded.

use rekall_kernel::jstr;

use crate::component::ComponentReferences;

use super::super::model::context_renderer::ContextRenderer;
use super::text::{truncate, MAX_DOCUMENT_CHARACTERS};

impl ContextRenderer {
    pub fn render_description(&self, description: Option<&str>, references: &mut ComponentReferences) -> String {
        match description {
            Some(text) if !jstr::is_blank(text) => {
                let expanded = references.expand(text);
                format!("\n<description>\n{}\n</description>\n", truncate(Some(&expanded), MAX_DOCUMENT_CHARACTERS))
            }
            _ => String::new(),
        }
    }
}
