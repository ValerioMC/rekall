use crate::worktree::StepCommit;

const SHORT_HASH_CHARACTERS: usize = 7;

/// The text of a pull request: the task's description, the issue it closes, and the commits listed
/// by step so the one-commit-per-step history shows. A part with nothing to say is left out.
pub fn pull_request_body(description: Option<&str>, issue_key: Option<&str>, commits: &[StepCommit]) -> String {
    let description = description.map(str::trim).filter(|text| !text.is_empty()).map(str::to_string);
    let closes = issue_key.map(|key| format!("Closes #{key}"));
    let listed = (!commits.is_empty()).then(|| format!("## Commits by step\n\n{}", commits.iter().map(commit_line).collect::<Vec<_>>().join("\n")));
    [description, closes, listed].into_iter().flatten().collect::<Vec<_>>().join("\n\n")
}

fn commit_line(commit: &StepCommit) -> String {
    let short_hash: String = commit.hash.chars().take(SHORT_HASH_CHARACTERS).collect();
    match &commit.step_title {
        Some(title) => format!("- **{title}**: `{short_hash}` {}", commit.subject),
        None => format!("- `{short_hash}` {}", commit.subject),
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/worktree/rule/pull_request_body_tests.rs"]
mod tests;
