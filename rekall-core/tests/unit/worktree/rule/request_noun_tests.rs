use super::*;

#[test]
fn gitlab_calls_it_a_merge_request_and_github_a_pull_request() {
    assert_eq!(request_noun(ConnectionKind::Gitlab), "merge request");
    assert_eq!(request_noun(ConnectionKind::Github), "pull request");
}
