use super::*;

const LISTING: &str = "worktree /work/vega\nHEAD 1111111\nbranch refs/heads/main\n\n\
worktree /data/worktrees/vega/report\nHEAD 2222222\nbranch refs/heads/rekall/report\n\n\
worktree /data/worktrees/vega/detached\nHEAD 3333333\ndetached\n\n\
worktree /elsewhere/feature\nHEAD 4444444\nbranch refs/heads/feature/x\n";

#[test]
fn only_the_worktrees_on_a_rekall_branch_are_listed() {
    let listed = parse_listing(Path::new("/work/vega"), LISTING);

    assert_eq!(
        listed,
        vec![Worktree {
            project_folder: PathBuf::from("/work/vega"),
            folder: PathBuf::from("/data/worktrees/vega/report"),
            branch: "rekall/report".into(),
        }]
    );
}

#[test]
fn an_empty_listing_has_no_worktree() {
    assert!(parse_listing(Path::new("/work/vega"), "").is_empty());
}
