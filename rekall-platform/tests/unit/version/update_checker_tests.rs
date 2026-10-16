use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;

use super::*;
use crate::version::{ReleaseAsset, ReleaseFeedError};

struct FakeFeed {
    answer: Result<Option<Release>, u16>,
    asked: AtomicUsize,
}

impl FakeFeed {
    fn answering(answer: Result<Option<Release>, u16>) -> Arc<Self> {
        Arc::new(Self { answer, asked: AtomicUsize::new(0) })
    }
}

#[async_trait]
impl ReleaseFeed for FakeFeed {
    async fn latest(&self) -> Result<Option<Release>, ReleaseFeedError> {
        self.asked.fetch_add(1, Ordering::SeqCst);
        self.answer.clone().map_err(ReleaseFeedError::Refused)
    }
}

fn release(tag: &str) -> Release {
    Release {
        tag: tag.to_string(),
        page_url: format!("https://example.test/releases/{tag}"),
        assets: vec![
            ReleaseAsset { name: "unrelated.txt".into(), download_url: "https://example.test/unrelated".into() },
            ReleaseAsset { name: format!("rekall{PLATFORM_ASSET_SUFFIX}"), download_url: "https://example.test/mine".into() },
        ],
    }
}

fn checker(running: &str, feed: Arc<FakeFeed>) -> UpdateChecker {
    UpdateChecker::new(running, true, feed)
}

#[tokio::test]
async fn a_newer_tag_is_an_update_with_the_asset_for_this_platform() {
    let status = checker("0.1.0", FakeFeed::answering(Ok(Some(release("v0.1.1"))))).status(false).await;
    assert_eq!(status.check, UpdateCheck::UpdateAvailable);
    assert_eq!(status.current, "0.1.0");
    let latest = status.latest.unwrap();
    assert_eq!(latest.version, "0.1.1");
    assert_eq!(latest.download_url.as_deref(), Some("https://example.test/mine"));
}

#[tokio::test]
async fn the_same_or_an_older_tag_is_up_to_date() {
    for tag in ["v0.1.1", "v0.1.0"] {
        let status = checker("0.1.1", FakeFeed::answering(Ok(Some(release(tag))))).status(false).await;
        assert_eq!(status.check, UpdateCheck::UpToDate, "{tag}");
    }
}

#[tokio::test]
async fn no_release_yet_is_up_to_date_with_nothing_to_offer() {
    let status = checker("0.1.0", FakeFeed::answering(Ok(None))).status(false).await;
    assert_eq!((status.check, status.latest), (UpdateCheck::UpToDate, None));
}

#[tokio::test]
async fn a_feed_that_fails_or_a_tag_that_is_not_a_version_is_unavailable() {
    let refused = checker("0.1.0", FakeFeed::answering(Err(503))).status(false).await;
    let unordered = checker("0.1.0", FakeFeed::answering(Ok(Some(release("latest"))))).status(false).await;
    assert_eq!(refused.check, UpdateCheck::Unavailable);
    assert_eq!(unordered.check, UpdateCheck::Unavailable);
}

#[tokio::test]
async fn the_answer_is_remembered_until_a_refresh_asks_again() {
    let feed = FakeFeed::answering(Ok(Some(release("v0.2.0"))));
    let checker = checker("0.1.0", feed.clone());
    checker.status(false).await;
    checker.status(false).await;
    assert_eq!(feed.asked.load(Ordering::SeqCst), 1);
    checker.status(true).await;
    assert_eq!(feed.asked.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn a_disabled_check_never_reaches_the_feed() {
    let feed = FakeFeed::answering(Ok(Some(release("v9.9.9"))));
    let status = UpdateChecker::new("0.1.0", false, feed.clone()).status(false).await;
    assert_eq!(status.check, UpdateCheck::Disabled);
    assert_eq!(feed.asked.load(Ordering::SeqCst), 0);
}
