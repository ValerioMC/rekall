//! The project's components: the folder a session opens to find each part.

use rekall_kernel::jstr;

use crate::component::ProjectComponentView;

use super::super::model::context_renderer::ContextRenderer;
use super::text::{attribute, truncate, MAX_DOCUMENT_CHARACTERS};

impl ContextRenderer {
    /// The project's parts: each folder a session can open to find that part, with what it is.
    pub fn render_components(&self, components: &[ProjectComponentView]) -> String {
        if components.is_empty() {
            return String::new();
        }
        let mut out = format!("\n<components count=\"{}\">\n", components.len());
        for component in components {
            out.push_str(&format!(
                "<component name=\"{}\" folder=\"{}\">\n",
                attribute(&component.name),
                attribute(&component.folder)
            ));
            if !jstr::is_blank(&component.description) {
                out.push_str(&truncate(Some(&component.description), MAX_DOCUMENT_CHARACTERS));
                out.push('\n');
            }
            out.push_str("</component>\n");
        }
        out.push_str("</components>\n");
        out
    }
}
