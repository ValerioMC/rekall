use super::super::scrollback::Scrollback;

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
