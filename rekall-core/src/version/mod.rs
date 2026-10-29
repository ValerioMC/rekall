//! The version this build is and the newest release, read from a feed that `rekall-infra`
//! implements over GitHub.

mod release_feed;
mod release_version;
mod version_status;

pub use release_feed::{Release, ReleaseAsset, ReleaseFeed, ReleaseFeedError};
pub use release_version::{InvalidVersion, ReleaseVersion, RUNNING_VERSION};
pub use version_status::{LatestRelease, UpdateCheck, VersionStatus};
