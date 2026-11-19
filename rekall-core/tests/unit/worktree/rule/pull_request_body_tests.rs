use super::*;

fn commit(step_title: Option<&str>, hash: &str, subject: &str) -> StepCommit {
    StepCommit { step_title: step_title.map(str::to_string), hash: hash.into(), subject: subject.into() }
}

#[test]
fn the_body_carries_the_description_the_issue_and_the_commits_by_step() {
    let commits = [commit(Some("Add the port"), "d044a3b1f2e9", "feat: Add the port"), commit(None, "f69b6d8aaaaa", "fix: Loose end")];

    let body = pull_request_body(Some("  Open pull requests.  "), Some("12"), &commits);

    assert_eq!(body, "Open pull requests.\n\nCloses #12\n\n## Commits by step\n\n- **Add the port**: `d044a3b` feat: Add the port\n- `f69b6d8` fix: Loose end");
}

#[test]
fn a_part_with_nothing_to_say_is_left_out() {
    assert_eq!(pull_request_body(None, None, &[]), "");
    assert_eq!(pull_request_body(Some("  "), Some("7"), &[]), "Closes #7");
}
