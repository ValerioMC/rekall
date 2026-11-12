//! The notes attached to the record: in full, or as a title and an opening line when they load on request.

use rekall_store::DocumentContextMode;

use crate::context::DocumentView;

use super::super::model::context_renderer::ContextRenderer;
use super::text::{opening_line, truncate, MAX_DOCUMENT_CHARACTERS};

impl ContextRenderer {
    pub fn render_documents(&self, documents: &[DocumentView]) -> String {
        let mut out = String::new();
        for document in documents {
            if document.context_mode == DocumentContextMode::Reference {
                out.push_str(&self.render_reference(document));
            } else {
                out.push_str(&format!(
                    "\n<document title=\"{}\" kind=\"{}\">\n{}\n</document>\n",
                    document.title,
                    document.kind,
                    truncate(Some(&document.body_markdown), MAX_DOCUMENT_CHARACTERS)
                ));
            }
        }
        out
    }

    /// A reference note: its title, its opening line so the session can judge whether it
    /// applies, and the anchor that loads the rest.
    pub fn render_reference(&self, document: &DocumentView) -> String {
        format!(
            "\n<document title=\"{}\" kind=\"{}\" anchor=\"{}\" loaded=\"on request\">\n{}\n\
             Not included. If the work needs it, load it with `rekall_context` and the anchor `{}`.\n\
             </document>\n",
            document.title,
            document.kind,
            document.anchor,
            opening_line(&document.body_markdown),
            document.anchor
        )
    }
}
