/// GitHub serves its API on a host of its own; a GitHub Enterprise Server serves it under `/api/v3`.
pub fn github_api_root(server_url: &str) -> String {
    let server = server_url.trim_end_matches('/');
    match server.strip_prefix("https://").or_else(|| server.strip_prefix("http://")) {
        Some("github.com" | "www.github.com") => "https://api.github.com".to_string(),
        _ => format!("{server}/api/v3"),
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/connection/model/github_api_root_tests.rs"]
mod tests;
