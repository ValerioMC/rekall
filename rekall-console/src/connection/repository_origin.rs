//! Where a repository's `origin` remote points, reduced to what matches it to a GitLab connection:
//! the host and the project path. Any user, password or token in the URL is dropped on parsing.

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RepositoryOrigin {
    host: String,
    path: String,
}

const SCHEMES: [&str; 6] = ["https", "http", "ssh", "git", "git+ssh", "ssh+git"];

impl RepositoryOrigin {
    /// Reads `https://host/group/name.git`, `ssh://git@host:2222/group/name.git` and git's short
    /// form `git@host:group/name.git`. A local path or another scheme is no origin to match.
    pub(super) fn parse(url: &str) -> Option<Self> {
        let url = url.trim();
        if let Some((scheme, rest)) = url.split_once("://") {
            if !SCHEMES.contains(&scheme.to_lowercase().as_str()) {
                return None;
            }
            let (authority, path) = rest.split_once('/')?;
            return Self::of(host_of(authority), path);
        }
        let (authority, path) = url.split_once(':')?;
        if authority.contains(['/', '\\']) {
            return None;
        }
        Self::of(host_of(authority), path)
    }

    fn of(host: &str, path: &str) -> Option<Self> {
        let path = path.trim_matches('/');
        let path = path.strip_suffix(".git").unwrap_or(path).trim_end_matches('/');
        if host.is_empty() || path.is_empty() {
            return None;
        }
        Some(Self { host: host.to_lowercase(), path: path.to_string() })
    }

    /// The project path on the GitLab instance at `base_url`, or none when the host differs. An
    /// instance served under a sub-path has it in an HTTPS origin but not in an SSH one.
    pub(super) fn path_on(&self, base_url: &str) -> Option<String> {
        let rest = base_url.split_once("://").map_or(base_url, |(_, rest)| rest);
        let (authority, prefix) = rest.split_once('/').unwrap_or((rest, ""));
        if host_of(authority).to_lowercase() != self.host {
            return None;
        }
        let prefix = prefix.trim_matches('/');
        if prefix.is_empty() {
            return Some(self.path.clone());
        }
        Some(self.path.strip_prefix(&format!("{prefix}/")).unwrap_or(&self.path).to_string())
    }

    /// `host/group/name`: what the console shows, with nothing secret left in it.
    pub(super) fn display(&self) -> String {
        format!("{}/{}", self.host, self.path)
    }
}

/// The host of an authority, without the user before `@` or the port after `:`; a bracketed IPv6
/// keeps its brackets.
fn host_of(authority: &str) -> &str {
    let host = authority.rsplit('@').next().unwrap_or(authority);
    if host.starts_with('[') {
        return host.find(']').map_or(host, |end| &host[..=end]);
    }
    host.split(':').next().unwrap_or(host)
}

#[cfg(test)]
#[path = "../../tests/unit/connection/repository_origin_tests.rs"]
mod tests;
