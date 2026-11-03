use super::*;

#[test]
fn files_and_rekall_folders_in_backticks_are_named() {
    let wrapup = "`WrapupService` is in `rekall-core/src/wrapup/service/wrapup_service.rs`; the UI is `rekall-ui` and its \
                  panel `rekall-ui/src/features/wrapup/components/WrapupVersionsPanel.vue`. Docs: `docs/DESIGN.md`.";

    assert_eq!(
        NamedPaths::in_text(wrapup),
        [
            "rekall-core/src/wrapup/service/wrapup_service.rs",
            "rekall-ui",
            "rekall-ui/src/features/wrapup/components/WrapupVersionsPanel.vue",
            "docs/DESIGN.md"
        ]
    );
}

#[test]
fn names_that_are_not_paths_are_left_out() {
    let wrapup = "`GET /api/tasks/{id}/check-results` and `rekall_context` and `project:vega task:x` and `.claude/rk/*.md` \
                  and `StepSnapshotService::take` and `https://example.com/a.md` and `/api/search.json` and `../x/y.rs` \
                  and `notes/readme` and `a/b.exe`.";

    assert!(NamedPaths::in_text(wrapup).is_empty(), "{:?}", NamedPaths::in_text(wrapup));
}

#[test]
fn a_path_named_twice_is_listed_once_and_one_outside_backticks_is_not_seen() {
    assert_eq!(NamedPaths::in_text("`a/b.rs` then `a/b.rs`, and a/c.rs without them"), ["a/b.rs"]);
}
