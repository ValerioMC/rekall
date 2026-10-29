use std::str::FromStr;

/// The version of this build: the tag the release workflow built from, or the crate's own
/// version for a local build.
pub const RUNNING_VERSION: &str = match option_env!("REKALL_VERSION") {
    Some(stamped) => stamped,
    None => env!("CARGO_PKG_VERSION"),
};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("`{0}` is not a version of the form MAJOR.MINOR.PATCH")]
pub struct InvalidVersion(pub String);

/// `MAJOR.MINOR.PATCH`, with or without the `v` a tag carries. Anything else, a pre-release
/// suffix included, is not a release this check can order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReleaseVersion {
    major: u64,
    minor: u64,
    patch: u64,
}

impl FromStr for ReleaseVersion {
    type Err = InvalidVersion;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let invalid = || InvalidVersion(text.to_string());
        let mut parts = text.trim().trim_start_matches('v').split('.');
        let mut next = || parts.next().and_then(|part| part.parse::<u64>().ok()).ok_or_else(invalid);
        let version = Self { major: next()?, minor: next()?, patch: next()? };
        if parts.next().is_some() {
            return Err(invalid());
        }
        Ok(version)
    }
}

#[cfg(test)]
#[path = "../../tests/unit/version/release_version_tests.rs"]
mod tests;
