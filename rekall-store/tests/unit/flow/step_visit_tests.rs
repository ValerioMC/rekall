use super::*;

#[test]
fn an_entered_visit_is_open_with_no_outcome() {
    let visit = Model::enter(Id::random(), Id::random(), 1, Instant::now());
    assert!(visit.is_open());
    assert!(visit.outcome.is_none());
}

#[test]
fn leaving_records_the_route_the_report_and_the_moment() {
    let mut visit = Model::enter(Id::random(), Id::random(), 1, Instant::now());
    let later = Instant::now().plus_seconds(5);
    visit.leave(Some("failed".into()), Some("Job `test` failed.".into()), later);
    assert!(!visit.is_open());
    assert_eq!(visit.outcome.as_deref(), Some("failed"));
    assert_eq!(visit.report.as_deref(), Some("Job `test` failed."));
    assert_eq!(visit.left_at, Some(later));
}

#[test]
fn leaving_a_closed_visit_keeps_the_first_route() {
    let mut visit = Model::enter(Id::random(), Id::random(), 1, Instant::now());
    visit.leave(Some("passed".into()), None, Instant::now());
    visit.leave(Some("app-error".into()), Some("late".into()), Instant::now().plus_seconds(1));
    assert_eq!(visit.outcome.as_deref(), Some("passed"));
    assert!(visit.report.is_none());
}
