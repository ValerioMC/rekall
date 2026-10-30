use std::sync::LazyLock;

use regex::Regex;

/// `setAllowedOriginPatterns`: `http://localhost`, `http://127.0.0.1` and `http://[::1]`, with
/// or without a port. A browser does not apply CORS to a WebSocket or to what an `EventSource`
/// reads from another origin it was allowed to ask, so these routes check it themselves.
pub fn is_allowed_origin(origin: &str) -> bool {
    static LOOPBACK: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^http://(localhost|127\.0\.0\.1|\[::1\])(:[0-9]*)?$").expect("a valid pattern"));
    LOOPBACK.is_match(&origin.to_lowercase())
}
