use super::*;

fn status(access: LocalAccess, peer: &str, host: &str, origin: Option<&str>) -> u16 {
    let peer: IpAddr = peer.parse().unwrap();
    if access.refusal(Some(peer), Some(host), origin).is_some() {
        403
    } else {
        200
    }
}

const LOCAL: LocalAccess = LocalAccess { remote_access: false };

#[test]
fn the_console_on_127_0_0_1_and_claude_code_on_localhost_both_pass() {
    assert_eq!(status(LOCAL, "127.0.0.1", "127.0.0.1:47355", Some("http://127.0.0.1:47355")), 200);
    assert_eq!(status(LOCAL, "::1", "localhost:47355", None), 200);
    assert_eq!(status(LOCAL, "127.0.0.1", "localhost:47355", Some("http://localhost:5173")), 200);
    assert_eq!(status(LOCAL, "::1", "[::1]:47355", Some("http://[::1]:47355")), 200);
    assert_eq!(status(LOCAL, "::ffff:127.0.0.1", "localhost:47355", None), 200, "an IPv4 client on a dual-stack socket");
}

#[test]
fn a_machine_on_the_same_network_is_refused_whatever_it_claims_in_its_headers() {
    assert_eq!(status(LOCAL, "192.168.1.20", "localhost:47355", None), 403);
    assert!(LOCAL.refusal(None, Some("localhost"), None).is_some(), "no peer address is no proof");
}

#[test]
fn a_host_that_is_not_this_machine_is_refused_that_is_dns_rebinding() {
    for host in ["evil.example:47355", "rebind.attacker.test", "192.168.1.5:47355"] {
        assert_eq!(status(LOCAL, "127.0.0.1", host, None), 403, "{host}");
    }
}

#[test]
fn an_origin_from_another_site_is_refused_that_is_a_page_in_the_same_browser() {
    for origin in ["https://evil.example", "null", "file://", "http://localhost.evil.example"] {
        assert_eq!(status(LOCAL, "127.0.0.1", "127.0.0.1:47355", Some(origin)), 403, "{origin}");
    }
}

#[test]
fn with_remote_access_on_another_machine_and_its_own_origin_pass_but_a_third_site_still_does_not() {
    let remote = LocalAccess { remote_access: true };
    assert_eq!(status(remote, "192.168.1.20", "192.168.1.10:47355", Some("http://192.168.1.10:47355")), 200);
    assert_eq!(status(remote, "192.168.1.20", "192.168.1.10:47355", Some("https://evil.example")), 403);
}

#[test]
fn the_host_is_read_without_its_port_bracketed_ipv6_included() {
    assert_eq!(host_name("localhost:47355"), "localhost");
    assert_eq!(host_name("[::1]:47355"), "[::1]");
    assert_eq!(host_name("127.0.0.1"), "127.0.0.1");
}
