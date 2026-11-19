//! The task's wrapup, with the warning when it has gone stale.

use crate::wrapup::{Freshness, WrapupView};

use super::super::model::context_renderer::ContextRenderer;
use super::text::{truncate, MAX_DOCUMENT_CHARACTERS};

impl ContextRenderer {
    pub fn render_wrapup(&self, wrapup: Option<&WrapupView>, freshness: &Freshness) -> String {
        match wrapup {
            None => String::new(),
            Some(wrapup) => format!(
                "\n<wrapup written-by=\"{}\" updated=\"{}\"{}>\n{}\n</wrapup>\n",
                wrapup.written_by,
                wrapup.updated_at,
                freshness.warning().map(|warning| format!(" freshness=\"{warning}\"")).unwrap_or_default(),
                truncate(Some(&wrapup.body_markdown), MAX_DOCUMENT_CHARACTERS)
            ),
        }
    }
}
