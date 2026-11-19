use rekall_store::external_connection;

/// Where a Jira or Confluence connection's calls go: the site, through the proxy the connection names when its
/// network reaches the site only that way.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtlassianSite {
    pub url: String,
    pub proxy_url: Option<String>,
}

impl AtlassianSite {
    pub fn of(connection: &external_connection::Model) -> Self {
        Self { url: connection.base_url.clone().unwrap_or_default(), proxy_url: connection.proxy_url.clone() }
    }
}
