use super::*;

#[test]
fn a_web_link_is_opened() {
    assert_eq!(web_link_of("https://github.com/acme/app/issues/12").unwrap().as_str(), "https://github.com/acme/app/issues/12");
    assert!(web_link_of("  http://gitlab.example.com/acme/app/-/issues/3 ").is_ok());
}

#[test]
fn anything_else_is_refused() {
    assert!(web_link_of("file:///etc/passwd").is_err());
    assert!(web_link_of("javascript:alert(1)").is_err());
    assert!(web_link_of("not a link").is_err());
}
