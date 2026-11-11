use super::*;

#[test]
fn a_blank_main_branch_means_the_folders_current_branch_and_a_name_is_trimmed() {
    assert_eq!(main_branch_of(None).unwrap(), None);
    assert_eq!(main_branch_of(Some("  ")).unwrap(), None);
    assert_eq!(main_branch_of(Some(" develop ")).unwrap().as_deref(), Some("develop"));
    assert_eq!(main_branch_of(Some("release/1.2")).unwrap().as_deref(), Some("release/1.2"));
}

#[test]
fn a_name_git_could_read_as_an_option_or_a_range_is_refused() {
    for name in ["-D", "main..other", "a b", "main~1", "x:y", "feature/", "/feature", "topic.lock", "a@{1}", "a\\b"] {
        assert!(main_branch_of(Some(name)).is_err(), "{name} was accepted");
    }
}
