//! The revisions of the description: open entries in full, applied ones as a line.

use rekall_store::FeedbackState;

use crate::feedback::FeedbackView;

use super::super::model::context_renderer::ContextRenderer;
use super::text::{opening_line, truncate, MAX_DOCUMENT_CHARACTERS};

impl ContextRenderer {
    /// The thread under the description: `PENDING` and `RUNNING` entries in full, which are what a
    /// session acts on, and `APPLIED` ones as one line each, which are history it never redoes.
    pub fn render_thread(&self, thread: &[FeedbackView]) -> String {
        if thread.is_empty() {
            return String::new();
        }
        let count = |state: FeedbackState| thread.iter().filter(|entry| entry.state == state).count();
        let mut out = format!(
            "\n<thread pending=\"{}\" running=\"{}\" applied=\"{}\">\n",
            count(FeedbackState::Pending),
            count(FeedbackState::Running),
            count(FeedbackState::Applied)
        );
        for entry in thread {
            out.push_str(&render_thread_entry(entry));
        }
        out.push_str("</thread>\n");
        out
    }
}

fn render_thread_entry(entry: &FeedbackView) -> String {
    let head = thread_entry_head(entry);
    if entry.state == FeedbackState::Applied {
        let reply = entry.reply_markdown.as_deref().map(opening_line).unwrap_or_default();
        return format!("<entry {head}>{reply}</entry>\n");
    }
    let body = truncate(Some(&entry.body_markdown), MAX_DOCUMENT_CHARACTERS);
    match entry.quoted_markdown.as_deref() {
        Some(quoted) => format!(
            "<entry {head}>\n<quoted>\n{}\n</quoted>\n<new>\n{body}\n</new>\n</entry>\n",
            truncate(Some(quoted), MAX_DOCUMENT_CHARACTERS)
        ),
        None => format!("<entry {head}>\n{body}\n</entry>\n"),
    }
}

fn thread_entry_head(entry: &FeedbackView) -> String {
    let mut head = format!("seq=\"{}\" kind=\"{}\" state=\"{}\"", entry.seq, entry.kind, entry.state);
    if let Some(pass) = entry.pass {
        head.push_str(&format!(" pass=\"{pass}\""));
    }
    head
}
