use super::*;

#[test]
fn the_scrollback_keeps_the_newest_bytes() {
    let mut ring = Scrollback::new(4);
    ring.append(b"ab");
    assert_eq!(ring.snapshot(), b"ab");
    ring.append(b"cdef");
    assert_eq!(ring.snapshot(), b"cdef");
    ring.append(b"g");
    assert_eq!(ring.snapshot(), b"defg");
}

#[test]
fn only_the_offered_model_and_effort_values_pass() {
    assert_eq!(normalise(Some(" Opus "), &MODEL_ALIASES).as_deref(), Some("opus"));
    assert_eq!(normalise(Some("gpt"), &MODEL_ALIASES), None);
    assert_eq!(normalise(None, &EFFORT_LEVELS), None);
}
