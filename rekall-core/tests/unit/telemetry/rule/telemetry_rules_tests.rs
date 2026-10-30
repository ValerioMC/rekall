use super::*;

#[test]
fn a_host_is_an_http_address_kept_without_its_trailing_slash() {
    assert_eq!(host(Some(" http://localhost:3000/ ")).unwrap(), "http://localhost:3000");
    assert_eq!(host(Some("https://langfuse.example.com")).unwrap(), "https://langfuse.example.com");
}

#[test]
fn a_host_that_is_missing_or_not_an_http_address_is_refused() {
    assert!(host(None).unwrap_err().message().contains("address"));
    assert!(host(Some("localhost:3000")).unwrap_err().message().contains("http://"));
    assert!(host(Some("http://local host")).unwrap_err().message().contains("no spaces"));
}

#[test]
fn a_public_key_may_be_empty_but_never_holds_spaces() {
    assert_eq!(public_key(None).unwrap(), "");
    assert_eq!(public_key(Some("  ")).unwrap(), "");
    assert_eq!(public_key(Some(" pk-lf-1 ")).unwrap(), "pk-lf-1");
    assert!(public_key(Some("pk lf")).unwrap_err().message().contains("no spaces"));
}

#[test]
fn a_secret_key_left_out_keeps_the_stored_one() {
    assert_eq!(secret_key(None).unwrap(), None);
    assert_eq!(secret_key(Some("  ")).unwrap(), None);
    assert_eq!(secret_key(Some("sk-lf-1")).unwrap(), Some("sk-lf-1".to_string()));
}

#[test]
fn a_secret_key_with_spaces_or_too_long_is_refused() {
    assert!(secret_key(Some("sk lf")).unwrap_err().message().contains("no spaces"));
    assert!(secret_key(Some(&"k".repeat(256))).unwrap_err().message().contains("at most 255"));
}
