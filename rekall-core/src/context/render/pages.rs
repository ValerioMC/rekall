//! The project's blueprint, and its pages listed by title.

use rekall_kernel::jstr;

use super::super::model::context_renderer::ContextRenderer;
use super::text::{truncate, MAX_DOCUMENT_CHARACTERS};

impl ContextRenderer {
    pub fn render_blueprint(&self, blueprint: Option<&str>) -> String {
        match blueprint {
            Some(text) if !jstr::is_blank(text) => {
                format!("\n<blueprint>\n{}\n</blueprint>\n", truncate(Some(text), MAX_DOCUMENT_CHARACTERS))
            }
            _ => String::new(),
        }
    }

    /// The project's pages by title only, so a session learns a procedure exists (how a change is
    /// proposed, how a release ships) without paying for every body in every context.
    pub fn render_sections(&self, titles: &[String]) -> String {
        if titles.is_empty() {
            return String::new();
        }
        let mut out = format!(
            "\n<sections count=\"{}\">\nPages of this project, not loaded here. When the work touches what a title names \
             (how a change is proposed, built, tested or released), read that page with `rekall_read_sections` and follow it.\n",
            titles.len()
        );
        for title in titles {
            out.push_str(&format!("- {title}\n"));
        }
        out.push_str("</sections>\n");
        out
    }
}
