#[derive(Clone, Debug)]
pub struct UpdateCheckConfig {
    /// `rekall.update-check.enabled`: asking GitHub whether a newer release exists.
    pub enabled: bool,
    /// `rekall.update-check.url`: the endpoint answering with the latest release.
    pub url: String,
}
