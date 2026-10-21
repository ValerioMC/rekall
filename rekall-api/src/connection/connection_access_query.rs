use serde::Deserialize;

/// Which page of a connection's access log, newest first.
#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ConnectionAccessQuery {
    pub offset: Option<u64>,
    pub limit: Option<u64>,
}
