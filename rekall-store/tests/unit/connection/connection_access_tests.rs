use super::*;

#[test]
fn a_granted_access_carries_no_reason() {
    let access = Model::granted(Id::random(), None, "terminal-1".into(), Instant::now());
    assert!(access.validate().is_ok());
    assert!(access.granted);
}

#[test]
fn a_refused_access_says_why() {
    let access = Model::refused(Id::random(), Some(Id::random()), "terminal-1".into(), "wrong token".into(), Instant::now());
    assert!(access.validate().is_ok());
    assert!(!access.granted);
}

#[test]
fn a_granted_access_with_a_reason_is_refused() {
    let mut access = Model::granted(Id::random(), None, "terminal-1".into(), Instant::now());
    access.reason = Some("why".into());
    assert!(access.validate().is_err());
}

#[test]
fn an_access_without_its_terminal_is_refused() {
    let access = Model::granted(Id::random(), None, " ".into(), Instant::now());
    assert!(access.validate().is_err());
}
