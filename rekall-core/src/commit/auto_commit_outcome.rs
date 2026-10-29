use super::CommitReferenceView;

use super::AutoCommitStatus;

/// What an automatic commit came to, for the session that triggered it.
#[derive(Clone, Debug)]
pub struct AutoCommitOutcome {
    pub status: AutoCommitStatus,
    pub detail: Option<String>,
    pub reference: Option<CommitReferenceView>,
}

impl AutoCommitOutcome {
    pub(super) fn not_applicable() -> Self {
        Self { status: AutoCommitStatus::Off, detail: None, reference: None }
    }

    pub(super) fn skipped(detail: String) -> Self {
        Self { status: AutoCommitStatus::Skipped, detail: Some(detail), reference: None }
    }

    pub(super) fn committed(reference: CommitReferenceView) -> Self {
        Self { status: AutoCommitStatus::Committed, detail: None, reference: Some(reference) }
    }

    pub(super) fn failed(detail: String) -> Self {
        Self { status: AutoCommitStatus::Failed, detail: Some(detail), reference: None }
    }

    pub fn off(&self) -> bool {
        self.status == AutoCommitStatus::Off
    }

    /// The line a tool appends to its report: nothing when the project does not auto-commit.
    pub fn describe(&self) -> String {
        let detail = self.detail.as_deref().unwrap_or("null");
        match self.status {
            AutoCommitStatus::Off => String::new(),
            AutoCommitStatus::Skipped => format!("Auto-commit: {detail}"),
            AutoCommitStatus::Failed => format!("Auto-commit failed: {detail} The claim stands; commit and log by hand."),
            AutoCommitStatus::Committed => {
                let reference = self.reference.as_ref().expect("a commit has its reference");
                let short = &reference.commit_hash[..reference.commit_hash.len().min(7)];
                let against = match &reference.step_title {
                    None => "the task".to_string(),
                    Some(title) => format!("\"{title}\""),
                };
                format!(
                    "Auto-committed `{short}` — {} — and logged it against {against}. Do not commit this work again.",
                    reference.comment
                )
            }
        }
    }
}
