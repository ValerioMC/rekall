//! Where the work departed from the plan, one entry per step.

use crate::deviation::DeviationView;

use super::super::model::context_renderer::ContextRenderer;
use super::text::{attribute, truncate, MAX_DOCUMENT_CHARACTERS};

impl ContextRenderer {
    pub fn render_deviations(&self, deviations: &[DeviationView]) -> String {
        if deviations.is_empty() {
            return String::new();
        }
        let mut out = format!("\n<deviations count=\"{}\">\n", deviations.len());
        for deviation in deviations {
            let title = deviation.step_title.as_deref().map(|title| format!(" title=\"{}\"", attribute(title))).unwrap_or_default();
            out.push_str(&format!(
                "<deviation step=\"{}\"{title}>\n{}\n</deviation>\n",
                deviation.step_number,
                truncate(Some(&deviation.body_markdown), MAX_DOCUMENT_CHARACTERS)
            ));
        }
        out.push_str("</deviations>\n");
        out
    }
}
