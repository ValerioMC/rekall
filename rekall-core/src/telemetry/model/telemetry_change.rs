/// What a person saves. A secret key left out keeps the one stored. It has no `Debug`, so the
/// key cannot reach a log.
pub struct TelemetryChange {
    pub enabled: bool,
    pub host: Option<String>,
    pub public_key: Option<String>,
    pub log_content: bool,
    pub secret_key: Option<String>,
}
