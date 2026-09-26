use super::*;

#[test]
fn only_a_loopback_page_may_open_the_pipe() {
    for allowed in ["http://localhost:5173", "http://127.0.0.1:47355", "http://[::1]:47355", "http://localhost"] {
        assert!(is_allowed_origin(allowed), "{allowed}");
    }
    for refused in ["https://evil.example", "http://localhost.evil.example", "https://localhost:47355"] {
        assert!(!is_allowed_origin(refused), "{refused}");
    }
}

#[test]
fn a_path_that_is_not_an_id_is_malformed() {
    assert!(terminal_id_of("not-an-id").is_none());
    assert!(terminal_id_of(&Id::random().to_string()).is_some());
}
