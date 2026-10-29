//! The version this build is and the newest release, read from a feed that `rekall-infra`
//! implements over GitHub.

mod model;
mod port;

pub use port::release_feed::{Release, ReleaseAsset, ReleaseFeed, ReleaseFeedError};
pub use model::release_version::{InvalidVersion, ReleaseVersion, RUNNING_VERSION};
pub use model::version_status::{LatestRelease, UpdateCheck, VersionStatus};
