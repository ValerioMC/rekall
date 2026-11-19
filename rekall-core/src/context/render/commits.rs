//! The commits a person chose to travel with the context, with their diffs.

use rekall_kernel::jstr;

use crate::context::ContextCommitView;

use super::super::model::context_renderer::ContextRenderer;
use super::text::{truncate, MAX_DIFF_CHARACTERS};

impl ContextRenderer {
    pub fn render_commits(&self, commits: &[ContextCommitView]) -> String {
        if commits.is_empty() {
            return String::new();
        }
        let mut out = format!("\n<commits count=\"{}\">\n", commits.len());
        for commit in commits {
            out.push_str(&format!("<commit hash=\"{}\"", commit.commit_hash));
            if let Some(step) = &commit.step_title {
                out.push_str(&format!(" step=\"{}\"", step.replace('"', "'")));
            }
            out.push_str(">\n");
            out.push_str(&commit.comment);
            out.push('\n');
            match commit.diff.as_deref() {
                Some(diff) if !jstr::is_blank(diff) => {
                    out.push('\n');
                    out.push_str(&truncate(Some(diff), MAX_DIFF_CHARACTERS));
                    out.push('\n');
                }
                _ => out.push_str("\n(no diff was recorded for this commit; read it from the repository by its hash)\n"),
            }
            out.push_str("</commit>\n");
        }
        out.push_str("</commits>\n");
        out
    }
}
