use axum::http::HeaderValue;

use super::*;

fn headers(pairs: &[(&'static str, &'static str)]) -> HeaderMap {
    let mut headers = HeaderMap::new();
    for (name, value) in pairs {
        headers.insert(*name, HeaderValue::from_static(value));
    }
    headers
}

#[test]
fn a_reconnecting_reader_resumes_from_the_last_event_it_saw() {
    assert_eq!(resume_after(&headers(&[("last-event-id", "41")]), Some(5)), 41);
}

#[test]
fn a_first_connection_resumes_from_the_query() {
    assert_eq!(resume_after(&headers(&[]), Some(5)), 5);
}

#[test]
fn nothing_asked_means_from_the_start() {
    assert_eq!(resume_after(&headers(&[]), None), 0);
    assert_eq!(resume_after(&headers(&[("last-event-id", "not a number")]), None), 0);
}

#[test]
fn only_a_loopback_origin_or_none_may_follow_a_session() {
    assert!(origin_is_allowed(&headers(&[])));
    assert!(origin_is_allowed(&headers(&[("origin", "http://localhost:5173")])));
    assert!(!origin_is_allowed(&headers(&[("origin", "https://evil.example")])));
}
