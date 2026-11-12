use super::*;
use crate::wrapup::{Freshness, WrapupView};
use rekall_kernel::{Id, Instant};
use rekall_store::WrapupAuthor;

fn wrapup() -> WrapupView {
    let now = Instant::now();
    WrapupView {
        id: Id::random(),
        task_id: Id::random(),
        task_label: "export".into(),
        task_title: "Export".into(),
        project_label: "vega".into(),
        anchor: "project:vega task:export".into(),
        body_markdown: "State.".into(),
        written_by: WrapupAuthor::Claude,
        created_at: now,
        updated_at: now,
    }
}

#[test]
fn a_possibly_stale_wrapup_carries_the_warning_on_its_tag_and_keeps_its_body() {
    let stale = Freshness::PossiblyStale { changed_paths: vec!["rekall-core/src/a.rs".into()], commits: vec!["c1".into(), "c2".into(), "c3".into()] };

    let rendered = ContextRenderer.render_wrapup(Some(&wrapup()), &stale);

    assert!(
        rendered.contains("freshness=\"possibly stale: rekall-core/src/a.rs changed in 3 commits since it was written\">"),
        "{rendered}"
    );
    assert!(rendered.contains("\nState.\n</wrapup>"));
}

#[test]
fn a_fresh_or_unknown_wrapup_renders_as_before() {
    for freshness in [Freshness::Fresh, Freshness::Unknown] {
        let rendered = ContextRenderer.render_wrapup(Some(&wrapup()), &freshness);
        assert!(!rendered.contains("freshness"), "{rendered}");
    }
}
