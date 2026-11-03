use crate::commit::CommitReferenceView;

use crate::commit::AutoCommitStatus;

/// What an automatic commit came to, for the session that triggered it. `left_out` names the
/// files that were already modified when the work started and so stayed out of the commit;
/// `snapshot_missing` says there was nothing to tell them apart by, so everything went in.
#[derive(Clone, Debug)]
pub struct AutoCommitOutcome {
    pub status: AutoCommitStatus,
    pub detail: Option<String>,
    pub reference: Option<CommitReferenceView>,
    pub left_out: Vec<String>,
    pub snapshot_missing: bool,
}

impl AutoCommitOutcome {
    pub(in crate::commit) fn not_applicable() -> Self {
        Self::of(AutoCommitStatus::Off, None, None)
    }

    pub(in crate::commit) fn skipped(detail: String) -> Self {
        Self::of(AutoCommitStatus::Skipped, Some(detail), None)
    }

    pub(in crate::commit) fn committed(reference: CommitReferenceView) -> Self {
        Self::of(AutoCommitStatus::Committed, None, Some(reference))
    }

    pub(in crate::commit) fn failed(detail: String) -> Self {
        Self::of(AutoCommitStatus::Failed, Some(detail), None)
    }

    fn of(status: AutoCommitStatus, detail: Option<String>, reference: Option<CommitReferenceView>) -> Self {
        Self { status, detail, reference, left_out: Vec::new(), snapshot_missing: false }
    }

    pub(in crate::commit) fn leaving_out(mut self, paths: Vec<String>) -> Self {
        self.left_out = paths;
        self
    }

    pub(in crate::commit) fn without_snapshot(mut self) -> Self {
        self.snapshot_missing = true;
        self
    }

    pub fn off(&self) -> bool {
        self.status == AutoCommitStatus::Off
    }

    /// The lines a tool appends to its report: nothing when the project does not auto-commit.
    pub fn describe(&self) -> String {
        let headline = self.headline();
        if headline.is_empty() {
            return headline;
        }
        let mut lines = vec![headline];
        if !self.left_out.is_empty() {
            lines.push(format!(
                "Left out because they were already modified before this work started: {}.",
                self.left_out.join(", ")
            ));
        }
        if self.snapshot_missing {
            lines.push("No snapshot of the folder from when this work started, so everything modified was committed.".to_string());
        }
        lines.join("\n")
    }

    fn headline(&self) -> String {
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
