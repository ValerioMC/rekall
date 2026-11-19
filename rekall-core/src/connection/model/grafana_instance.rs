use rekall_store::external_connection;

/// Where a Grafana connection's calls go: the instance, through the proxy the connection names, and
/// whether its self-signed certificate is trusted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrafanaInstance {
    pub url: String,
    pub proxy_url: Option<String>,
    pub accept_self_signed: bool,
}

impl GrafanaInstance {
    pub fn of(connection: &external_connection::Model) -> Self {
        Self {
            url: connection.base_url.clone().unwrap_or_default(),
            proxy_url: connection.proxy_url.clone(),
            accept_self_signed: connection.accept_self_signed,
        }
    }
}
