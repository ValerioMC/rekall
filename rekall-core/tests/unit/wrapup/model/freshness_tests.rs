use super::*;

#[test]
fn only_a_possibly_stale_wrapup_has_a_warning() {
    assert_eq!(Freshness::Fresh.warning(), None);
    assert_eq!(Freshness::Unknown.warning(), None);
}

#[test]
fn the_warning_names_up_to_three_files_and_counts_the_commits() {
    let one = Freshness::PossiblyStale { changed_paths: vec!["a/b.rs".into()], commits: vec!["c1".into()] };
    let many = Freshness::PossiblyStale {
        changed_paths: vec!["a.rs".into(), "b.rs".into(), "c.rs".into(), "d.rs".into()],
        commits: vec!["c1".into(), "c2".into(), "c3".into()],
    };

    assert_eq!(one.warning().as_deref(), Some("possibly stale: a/b.rs changed in 1 commit since it was written"));
    assert_eq!(many.warning().as_deref(), Some("possibly stale: a.rs, b.rs, c.rs, … changed in 3 commits since it was written"));
}
