//! Turns loaded context records into the markdown a session reads.

use std::collections::HashSet;

use rekall_common::jstr;
use rekall_model::{DocumentContextMode, TaskStepState};

use super::{ContextCommitView, ContextRecord, DocumentView};
use crate::step::TaskStepView;
use crate::wrapup::WrapupView;

pub const MAX_DOCUMENT_CHARACTERS: usize = 20_000;

/// A diff is the one thing here worth more room than a note: it is what the session was handed to read.
pub const MAX_DIFF_CHARACTERS: usize = 60_000;

/// How much of a reference note's opening line travels with it.
pub const REFERENCE_LINE_MAX: usize = 200;

#[derive(Clone, Copy, Debug, Default)]
pub struct ContextRenderer;

impl ContextRenderer {
    /// The whole answer to one `rekall_context` call: every record once, in the order loaded.
    pub fn render(&self, records: &[ContextRecord]) -> String {
        let mut out = String::from("# Context\n");
        let mut rendered = HashSet::new();
        for record in records {
            out.push('\n');
            out.push_str(&self.render_record(record, 2, &mut rendered));
        }
        out
    }

    pub fn render_record(&self, record: &ContextRecord, heading_level: usize, rendered: &mut HashSet<String>) -> String {
        if !rendered.insert(record.anchor.clone()) {
            return String::new();
        }
        let mut out = format!(
            "{} {}: {}\n\n- `anchor`: `{}`\n",
            "#".repeat(heading_level),
            record.kind,
            record.label,
            record.anchor
        );
        for (key, value) in &record.fields {
            out.push_str(&format!("- `{key}`: {value}\n"));
        }
        if !record.related.is_empty() {
            out.push_str("- `related`:\n");
            for anchor in &record.related {
                out.push_str(&format!("  - `{anchor}`\n"));
            }
        }
        out.push_str(&self.render_description(record.description.as_deref()));
        out.push_str(&self.render_blueprint(record.blueprint.as_deref()));
        out.push_str(&self.render_steps(&record.steps, record.wrapup.as_ref()));
        out.push_str(&self.render_commits(&record.commits));
        out.push_str(&self.render_wrapup(record.wrapup.as_ref()));
        out.push_str(&self.render_documents(&record.documents));
        for reference in &record.references {
            let nested = self.render_record(reference, heading_level + 1, rendered);
            if !nested.is_empty() {
                out.push('\n');
                out.push_str(&nested);
            }
        }
        out
    }

    pub fn render_steps(&self, all_steps: &[TaskStepView], wrapup: Option<&WrapupView>) -> String {
        let drafts = all_steps.iter().filter(|s| s.state.draft()).count();
        let steps: Vec<&TaskStepView> = all_steps.iter().filter(|s| !s.state.draft()).collect();
        if steps.is_empty() {
            return String::new();
        }
        let done = steps.iter().filter(|s| s.state.complete()).count();
        let running = steps.iter().filter(|s| s.state.running()).count();
        let awaiting = steps.iter().filter(|s| s.state == TaskStepState::Claimed).count();
        let unwritten = steps.iter().filter(|s| is_unwritten(s, wrapup)).count();

        let mut attributes = format!("done=\"{done}\" open=\"{}\"", steps.len() - done);
        if running > 0 {
            attributes.push_str(&format!(" running=\"{running}\""));
        }
        if awaiting > 0 {
            attributes.push_str(&format!(" awaiting-review=\"{awaiting}\""));
        }
        if unwritten > 0 {
            attributes.push_str(&format!(" finished-since-wrapup=\"{unwritten}\""));
        }
        if drafts > 0 {
            attributes.push_str(&format!(" draft=\"{drafts}\""));
        }
        let mut out = format!("\n<steps {attributes}>\n");
        for step in &steps {
            let unwritten_here = is_unwritten(step, wrapup);
            out.push_str(if step.state.complete() { "- [x] " } else { "- [ ] " });
            out.push_str(&step.title);
            if step.state.running() {
                out.push_str("  (in progress)");
            } else if step.state == TaskStepState::Claimed {
                out.push_str("  (claimed, waiting for the console to accept it)");
            }
            if unwritten_here {
                out.push_str("  (finished since the wrapup was written)");
            }
            out.push('\n');
            let carries_detail = !step.state.complete() || unwritten_here;
            if let Some(body) = step.body_markdown.as_deref() {
                if carries_detail && !jstr::is_blank(body) {
                    out.push_str(&indent(&truncate(Some(body), MAX_DOCUMENT_CHARACTERS)));
                    out.push('\n');
                }
            }
        }
        if drafts > 0 {
            out.push_str(&format!(
                "<!-- {drafts} further step{} still in draft, not shown: promoted to the checklist \
                 in the console when it is ready to work -->\n",
                if drafts == 1 { "" } else { "s" }
            ));
        }
        out.push_str("</steps>\n");
        out
    }

    pub fn render_wrapup(&self, wrapup: Option<&WrapupView>) -> String {
        match wrapup {
            None => String::new(),
            Some(wrapup) => format!(
                "\n<wrapup written-by=\"{}\" updated=\"{}\">\n{}\n</wrapup>\n",
                wrapup.written_by,
                wrapup.updated_at,
                truncate(Some(&wrapup.body_markdown), MAX_DOCUMENT_CHARACTERS)
            ),
        }
    }

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

    pub fn render_description(&self, description: Option<&str>) -> String {
        match description {
            Some(text) if !jstr::is_blank(text) => {
                format!("\n<description>\n{}\n</description>\n", truncate(Some(text), MAX_DOCUMENT_CHARACTERS))
            }
            _ => String::new(),
        }
    }

    pub fn render_blueprint(&self, blueprint: Option<&str>) -> String {
        match blueprint {
            Some(text) if !jstr::is_blank(text) => {
                format!("\n<blueprint>\n{}\n</blueprint>\n", truncate(Some(text), MAX_DOCUMENT_CHARACTERS))
            }
            _ => String::new(),
        }
    }

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

fn is_unwritten(step: &TaskStepView, wrapup: Option<&WrapupView>) -> bool {
    if !step.state.complete() {
        return false;
    }
    let Some(completed) = step.completed_at() else {
        return false;
    };
    match wrapup {
        None => true,
        Some(wrapup) => completed.is_after(&wrapup.updated_at),
    }
}

fn indent(body: &str) -> String {
    jstr::lines(body)
        .into_iter()
        .map(|line| if jstr::is_blank(line) { String::new() } else { format!("  {line}") })
        .collect::<Vec<_>>()
        .join("\n")
}

fn opening_line(body: &str) -> String {
    let first = jstr::lines(body)
        .into_iter()
        .map(jstr::strip)
        .find(|line| !line.is_empty() && !line.starts_with('#') && !line.starts_with("```"))
        .unwrap_or("");
    if jstr::len(first) <= REFERENCE_LINE_MAX {
        first.to_string()
    } else {
        format!("{}…", jstr::strip_trailing(jstr::prefix(first, REFERENCE_LINE_MAX)))
    }
}

fn truncate(body: Option<&str>, limit: usize) -> String {
    let Some(body) = body else {
        return String::new();
    };
    let length = jstr::len(body);
    if length <= limit {
        return body.to_string();
    }
    format!("{}\n\n[truncated: {limit} of {length} characters shown]", jstr::prefix(body, limit))
}
