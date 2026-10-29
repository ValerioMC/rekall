use std::sync::Arc;
use std::time::{Duration, Instant};

use rekall_core::version::{LatestRelease, Release, ReleaseFeed, ReleaseVersion, UpdateCheck, VersionStatus};
use tokio::sync::Mutex;
use tracing::warn;

const ANSWER_KEPT: Duration = Duration::from_secs(6 * 60 * 60);
const FAILURE_KEPT: Duration = Duration::from_secs(5 * 60);

/// What names this platform's asset in a release (see `release.yml`).
const PLATFORM_ASSET_SUFFIX: &str = if cfg!(target_os = "macos") {
    ".dmg"
} else if cfg!(target_os = "windows") {
    "-windows-x64.exe"
} else {
    "-linux-x64"
};

struct Remembered {
    at: Instant,
    kept_for: Duration,
    status: VersionStatus,
}

/// Compares the running version with the newest release. The answer is remembered, so opening
/// the console repeatedly asks GitHub once; a restart forgets it, which is the check at startup.
#[derive(Clone)]
pub struct UpdateCheckService {
    running: String,
    feed: Option<Arc<dyn ReleaseFeed>>,
    remembered: Arc<Mutex<Option<Remembered>>>,
}

impl UpdateCheckService {
    /// Without a feed the check is turned off and never reaches the network.
    pub fn new(running: &str, feed: Option<Arc<dyn ReleaseFeed>>) -> Self {
        Self { running: running.to_string(), feed, remembered: Arc::new(Mutex::new(None)) }
    }

    /// `refresh` ignores what is remembered: the person asked, not the console loading.
    pub async fn status(&self, refresh: bool) -> VersionStatus {
        let Some(feed) = &self.feed else {
            return self.answer(UpdateCheck::Disabled, None);
        };
        let mut remembered = self.remembered.lock().await;
        if let Some(kept) = remembered.as_ref().filter(|kept| !refresh && kept.at.elapsed() < kept.kept_for) {
            return kept.status.clone();
        }
        let (status, kept_for) = self.ask(feed.as_ref()).await;
        *remembered = Some(Remembered { at: Instant::now(), kept_for, status: status.clone() });
        status
    }

    async fn ask(&self, feed: &dyn ReleaseFeed) -> (VersionStatus, Duration) {
        match feed.latest().await {
            Ok(None) => (self.answer(UpdateCheck::UpToDate, None), ANSWER_KEPT),
            Ok(Some(release)) => match self.compare(&release) {
                Some(check) => (self.answer(check, Some(latest_of(&release))), ANSWER_KEPT),
                None => (self.answer(UpdateCheck::Unavailable, None), FAILURE_KEPT),
            },
            Err(failure) => {
                warn!(%failure, "Could not check for a newer release");
                (self.answer(UpdateCheck::Unavailable, None), FAILURE_KEPT)
            }
        }
    }

    fn compare(&self, release: &Release) -> Option<UpdateCheck> {
        let running = self.running.parse::<ReleaseVersion>().ok()?;
        let latest = release.tag.parse::<ReleaseVersion>().ok()?;
        Some(if latest > running { UpdateCheck::UpdateAvailable } else { UpdateCheck::UpToDate })
    }

    fn answer(&self, check: UpdateCheck, latest: Option<LatestRelease>) -> VersionStatus {
        VersionStatus { current: self.running.clone(), check, latest }
    }
}

fn latest_of(release: &Release) -> LatestRelease {
    LatestRelease {
        version: release.tag.trim_start_matches('v').to_string(),
        release_url: release.page_url.clone(),
        download_url: release
            .assets
            .iter()
            .find(|asset| asset.name.ends_with(PLATFORM_ASSET_SUFFIX))
            .map(|asset| asset.download_url.clone()),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/version/update_check_service_tests.rs"]
mod tests;
