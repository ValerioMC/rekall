//! `LocalAccessFilter`: keeps Rekall a single-user, localhost application. Nothing here
//! authenticates, and the API can open a `claude` terminal in a project folder, so every request
//! has to prove it comes from this machine and from a page this machine serves. Three checks, all
//! refused with a 403:
//!
//! - The peer address is loopback. The server listens on every interface so that both
//!   `127.0.0.1` and `::1` answer whichever one a client resolves `localhost` to; this is what
//!   stops a machine on the same network.
//! - The `Host` header names a loopback host. A page on another domain that rebinds its DNS to
//!   `127.0.0.1` still sends its own name here, so this is what stops DNS rebinding.
//! - An `Origin` header, when there is one, is a loopback origin. A browser sends it on every
//!   cross-site write and WebSocket handshake, so this is what stops another site open in the same
//!   browser from driving the API. Clients that send none (Claude Code, curl) pass.
//!
//! `rekall.security.remote-access=true` turns the first two off for someone who deliberately
//! serves Rekall to another machine. The origin check then also accepts the page's own origin,
//! the one its `Host` names, so the console served that way still works; that mode gives up the
//! protection against DNS rebinding and is off by default.

use std::net::{IpAddr, SocketAddr};
use std::sync::LazyLock;

use axum::extract::{ConnectInfo, Request, State};
use axum::http::{header, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use regex::Regex;
use rekall_api::error::framework;
use tracing::warn;

const LOOPBACK_NAMES: [&str; 4] = ["localhost", "127.0.0.1", "::1", "[::1]"];
static IPV4_LOOPBACK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^127(\.\d{1,3}){3}$").unwrap());

#[derive(Clone, Copy, Debug)]
pub struct LocalAccess {
    pub remote_access: bool,
}

impl LocalAccess {
    pub fn new(remote_access: bool) -> Self {
        if remote_access {
            warn!("rekall.security.remote-access is on: requests from other machines are accepted");
        }
        Self { remote_access }
    }

    /// Why a request is refused, or `None` to let it through.
    pub fn refusal(&self, peer: Option<IpAddr>, host: Option<&str>, origin: Option<&str>) -> Option<String> {
        if !self.remote_access && !peer.is_some_and(|ip| ip.to_canonical().is_loopback()) {
            return Some("Rekall only answers requests from this machine".into());
        }
        if !self.remote_access {
            if let Some(host) = host {
                if !is_loopback_name(&host_name(host)) {
                    return Some(format!("Host '{host}' is not this machine"));
                }
            }
        }
        if let Some(origin) = origin {
            let this_machine = is_loopback_origin(origin) || (self.remote_access && is_same_origin(origin, host));
            if !this_machine {
                return Some(format!("Origin '{origin}' is not a page this machine serves"));
            }
        }
        None
    }
}

pub async fn guard(State(access): State<LocalAccess>, request: Request, next: Next) -> Response {
    let refusal = {
        let peer = request.extensions().get::<ConnectInfo<SocketAddr>>().map(|info| info.0.ip());
        let host = request.headers().get(header::HOST).and_then(|v| v.to_str().ok());
        let origin = request.headers().get(header::ORIGIN).and_then(|v| v.to_str().ok());
        access.refusal(peer, host, origin).map(|refusal| {
            format!(
                "Refused {} {} from {}: {refusal}",
                request.method(),
                request.uri().path(),
                peer.map(|p| p.to_string()).unwrap_or_default()
            )
        })
    };
    if let Some(refusal) = refusal {
        warn!("{refusal}");
        return framework(StatusCode::FORBIDDEN);
    }
    next.run(request).await
}

/// The part of a `Host` header before its port, bracketed IPv6 included.
pub fn host_name(host_header: &str) -> String {
    let value = host_header.trim();
    if value.starts_with('[') {
        return match value.find(']') {
            Some(close) => value[..=close].to_string(),
            None => value.to_string(),
        };
    }
    match value.rfind(':') {
        Some(colon) => value[..colon].to_string(),
        None => value.to_string(),
    }
}

fn is_loopback_name(host: &str) -> bool {
    let name = host.to_lowercase();
    LOOPBACK_NAMES.contains(&name.as_str()) || IPV4_LOOPBACK.is_match(&name)
}

/// `scheme://authority/...` split as `java.net.URI` would, for the two parts the checks need.
struct Origin {
    scheme: String,
    authority: String,
    host: Option<String>,
}

fn parse_origin(origin: &str) -> Option<Origin> {
    let origin = origin.trim();
    if origin.chars().any(|c| c.is_whitespace() || c == '"' || c == '<' || c == '>') {
        return None;
    }
    let (scheme, rest) = origin.split_once("://")?;
    if scheme.is_empty() || !scheme.chars().all(|c| c.is_ascii_alphanumeric() || "+-.".contains(c)) {
        return None;
    }
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("").to_string();
    let host_port = authority.rsplit_once('@').map(|(_, h)| h).unwrap_or(&authority);
    let host = if host_port.starts_with('[') {
        host_port.find(']').map(|close| host_port[..=close].to_string())
    } else {
        Some(host_port.split(':').next().unwrap_or("").to_string())
    };
    Some(Origin { scheme: scheme.to_string(), authority, host: host.filter(|h| !h.is_empty()) })
}

pub fn is_loopback_origin(origin: &str) -> bool {
    let Some(parsed) = parse_origin(origin) else { return false };
    let scheme = parsed.scheme.to_lowercase();
    if scheme != "http" && scheme != "https" {
        return false;
    }
    parsed.host.is_some_and(|host| is_loopback_name(&host))
}

fn is_same_origin(origin: &str, host_header: Option<&str>) -> bool {
    let Some(host) = host_header else { return false };
    parse_origin(origin).is_some_and(|parsed| host.trim().eq_ignore_ascii_case(&parsed.authority))
}

#[cfg(test)]
mod tests {
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
}
