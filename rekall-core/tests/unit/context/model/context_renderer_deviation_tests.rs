use super::*;

fn deviation(step_number: i32, step_title: Option<&str>, body: &str) -> DeviationView {
    DeviationView { step_number, step_title: step_title.map(str::to_string), body_markdown: body.into() }
}

#[test]
fn deviations_are_listed_apart_from_the_notes_with_the_step_each_refers_to() {
    let rendered = ContextRenderer.render_deviations(&[
        deviation(2, Some("Wire \"the\" service"), "Built it in core instead."),
        deviation(5, None, "Dropped the cache."),
    ]);

    assert!(rendered.contains("<deviations count=\"2\">"), "{rendered}");
    assert!(rendered.contains("<deviation step=\"2\" title=\"Wire 'the' service\">\nBuilt it in core instead.\n</deviation>"), "{rendered}");
    assert!(rendered.contains("<deviation step=\"5\">\nDropped the cache.\n</deviation>"), "{rendered}");
    assert!(!rendered.contains("<document"), "{rendered}");
}

#[test]
fn a_task_without_deviations_renders_nothing() {
    assert_eq!(ContextRenderer.render_deviations(&[]), "");
}
