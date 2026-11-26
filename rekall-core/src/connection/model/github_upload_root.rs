use crate::connection::github_api_root;

/// The host `gh` sends attachments to. A GitHub Enterprise Server has no such endpoint.
pub fn github_upload_root(server_url: &str) -> Option<&'static str> {
    (github_api_root(server_url) == "https://api.github.com").then_some("https://uploads.github.com")
}

#[cfg(test)]
#[path = "../../../tests/unit/connection/model/github_upload_root_tests.rs"]
mod tests;
