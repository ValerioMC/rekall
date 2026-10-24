//! The inventory a session writes, the scan that matches it against critical advisories through a
//! fake feed, and the assessment a session gives an exposure.

mod support;

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use rekall_common::Instant;
use rekall_model::{Ecosystem, ExposureVerdict};
use rekall_service::inventory::DeclaredDependency;
use rekall_service::vulnerability::{AdvisoryFeed, AdvisoryFeedFailure, AffectedPackage, PackageVersion, PublishedAdvisory, VulnerabilityScanService};
use support::{world, World};

/// What the feed answers one question with.
type Answer = Result<Vec<PublishedAdvisory>, AdvisoryFeedFailure>;

/// Answers every question about an ecosystem with what the test set for it, and remembers the questions.
#[derive(Default)]
struct FakeFeed {
    latest: Vec<PublishedAdvisory>,
    affecting: Mutex<Vec<(Ecosystem, Answer)>>,
    asked: Mutex<Vec<(Ecosystem, Vec<PackageVersion>)>>,
}

impl FakeFeed {
    fn answering(ecosystem: Ecosystem, answer: Answer) -> Self {
        Self { affecting: Mutex::new(vec![(ecosystem, answer)]), ..Self::default() }
    }

    fn answer(&self, ecosystem: Ecosystem, answer: Answer) {
        let mut affecting = self.affecting.lock().unwrap();
        affecting.retain(|(known, _)| *known != ecosystem);
        affecting.push((ecosystem, answer));
    }
}

#[async_trait]
impl AdvisoryFeed for FakeFeed {
    async fn latest_critical(&self, _limit: u16) -> Result<Vec<PublishedAdvisory>, AdvisoryFeedFailure> {
        Ok(self.latest.clone())
    }

    async fn critical_affecting(&self, ecosystem: Ecosystem, packages: &[PackageVersion]) -> Answer {
        self.asked.lock().unwrap().push((ecosystem, packages.to_vec()));
        let affecting = self.affecting.lock().unwrap();
        affecting.iter().find(|(known, _)| *known == ecosystem).map(|(_, answer)| answer.clone()).unwrap_or(Ok(Vec::new()))
    }
}

fn advisory(ghsa_id: &str, ecosystem: Ecosystem, name: &str) -> PublishedAdvisory {
    PublishedAdvisory {
        ghsa_id: ghsa_id.into(),
        cve_id: Some("CVE-2026-0001".into()),
        summary: format!("Remote code execution in {name}"),
        description: "An attacker runs code.".into(),
        url: format!("https://github.com/advisories/{ghsa_id}"),
        published_at: Instant::now(),
        packages: vec![AffectedPackage {
            ecosystem,
            name: name.into(),
            vulnerable_range: Some("< 2.0.0".into()),
            first_patched: Some("2.0.0".into()),
            vulnerable_functions: vec![format!("{name}.parse")],
        }],
    }
}

fn declared(ecosystem: &str, name: &str, version: &str) -> DeclaredDependency {
    DeclaredDependency { ecosystem: Some(ecosystem.into()), name: Some(name.into()), version: Some(version.into()), manifest: Some("package-lock.json".into()) }
}

fn scanner(world: &World, feed: Arc<FakeFeed>) -> VulnerabilityScanService {
    let services = &world.services;
    VulnerabilityScanService::new(services.ctx.clone(), feed, services.inventory.clone(), services.advisories.clone(), services.exposures.clone())
}

#[tokio::test]
async fn a_scan_asks_about_each_inventoried_package_and_records_what_reaches_it() {
    let world = world().await;
    let vega = world.project("vega", None, false).await;
    world.project("idle", None, false).await;
    world.services.inventory_recording.record("vega", &[declared("npm", "Left-Pad", "1.0.0"), declared("pip", "django", "4.2.0")]).await.unwrap();
    let feed = Arc::new(FakeFeed { latest: vec![advisory("GHSA-aaaa-aaaa-aaaa", Ecosystem::Maven, "log4j")], ..FakeFeed::answering(Ecosystem::Npm, Ok(vec![advisory("GHSA-bbbb-bbbb-bbbb", Ecosystem::Npm, "left-pad")])) });

    let report = scanner(&world, feed.clone()).scan().await.unwrap();

    assert_eq!((report.advisories, report.projects_scanned, report.new_exposures, report.open_exposures), (2, 1, 1, 1));
    assert!(report.failures.is_empty());
    let asked = feed.asked.lock().unwrap().clone();
    assert_eq!(asked.len(), 2, "one question per ecosystem: {asked:?}");
    let exposures = world.services.exposures.list().await.unwrap();
    assert_eq!(exposures.len(), 1);
    let exposure = &exposures[0];
    assert_eq!((exposure.project_id, exposure.package_name.as_str(), exposure.package_version.as_str()), (vega.id, "Left-Pad", "1.0.0"));
    assert_eq!(exposure.first_patched.as_deref(), Some("2.0.0"));
    assert!(exposure.open);
    assert_eq!(world.services.advisories.latest(10).await.unwrap().len(), 2);
}

#[tokio::test]
async fn a_scan_keeps_a_verdict_on_an_exposure_found_again_and_drops_one_no_longer_found() {
    let world = world().await;
    world.project("vega", None, false).await;
    world.services.inventory_recording.record("vega", &[declared("npm", "left-pad", "1.0.0")]).await.unwrap();
    let feed = Arc::new(FakeFeed::answering(Ecosystem::Npm, Ok(vec![advisory("GHSA-bbbb-bbbb-bbbb", Ecosystem::Npm, "left-pad")])));
    let scan = scanner(&world, feed.clone());
    scan.scan().await.unwrap();
    world.services.exposures.assess("vega", Some("ghsa-bbbb-bbbb-bbbb"), Some("not_affected"), Some("Only `pad` is called, never `parse`."), Some("Vega pads strings and parses none.")).await.unwrap();

    let again = scan.scan().await.unwrap();
    assert_eq!((again.new_exposures, again.open_exposures), (0, 0));
    let kept = world.services.exposures.list().await.unwrap();
    assert_eq!(kept[0].verdict, Some(ExposureVerdict::NotAffected));
    assert!(!kept[0].open);

    feed.answer(Ecosystem::Npm, Ok(Vec::new()));
    scan.scan().await.unwrap();
    assert!(world.services.exposures.list().await.unwrap().is_empty());
}

#[tokio::test]
async fn a_project_the_feed_could_not_answer_for_keeps_its_exposures_and_the_rest_wait_for_the_limit() {
    let world = world().await;
    world.project("alpha", None, false).await;
    world.project("beta", None, false).await;
    world.services.inventory_recording.record("alpha", &[declared("npm", "left-pad", "1.0.0")]).await.unwrap();
    world.services.inventory_recording.record("beta", &[declared("npm", "left-pad", "1.0.0")]).await.unwrap();
    let feed = Arc::new(FakeFeed::answering(Ecosystem::Npm, Ok(vec![advisory("GHSA-bbbb-bbbb-bbbb", Ecosystem::Npm, "left-pad")])));
    let scan = scanner(&world, feed.clone());
    scan.scan().await.unwrap();

    feed.answer(Ecosystem::Npm, Err(AdvisoryFeedFailure::RateLimited));
    let report = scan.scan().await.unwrap();

    assert_eq!(report.projects_scanned, 0);
    assert_eq!(report.failures.len(), 2, "{:?}", report.failures);
    assert!(report.failures[0].contains("`project:alpha` was not checked") && report.failures[0].contains("keeps the exposures it had"));
    assert!(report.failures[1].contains("`project:beta` was not checked"));
    assert_eq!(feed.asked.lock().unwrap().len(), 3, "beta is not asked once the limit is reached");
    assert_eq!(world.services.exposures.list().await.unwrap().len(), 2);
}

#[tokio::test]
async fn an_inventory_without_the_package_settles_its_exposure_at_once() {
    let world = world().await;
    world.project("vega", None, false).await;
    world.services.inventory_recording.record("vega", &[declared("npm", "left-pad", "1.0.0")]).await.unwrap();
    let feed = Arc::new(FakeFeed::answering(Ecosystem::Npm, Ok(vec![advisory("GHSA-bbbb-bbbb-bbbb", Ecosystem::Npm, "left-pad")])));
    scanner(&world, feed).scan().await.unwrap();

    let recorded = world.services.inventory_recording.record("vega", &[declared("npm", "left-pad", "2.0.0"), declared("npm", "left-pad", "2.0.0")]).await.unwrap();

    assert_eq!((recorded.written.stored, recorded.written.repeats, recorded.settled_exposures), (1, 1, 1));
    assert!(world.services.exposures.list().await.unwrap().is_empty());
    assert_eq!(world.services.inventory.list(recorded.written.project_id).await.unwrap()[0].version, "2.0.0");
}

#[tokio::test]
async fn an_assessment_names_an_advisory_that_reaches_the_project() {
    let world = world().await;
    world.project("vega", None, false).await;
    world.services.inventory_recording.record("vega", &[declared("npm", "left-pad", "1.0.0")]).await.unwrap();
    let feed = Arc::new(FakeFeed::answering(Ecosystem::Npm, Ok(vec![advisory("GHSA-bbbb-bbbb-bbbb", Ecosystem::Npm, "left-pad")])));
    scanner(&world, feed).scan().await.unwrap();

    let refused = world.services.exposures.assess("vega", Some("GHSA-cccc-cccc-cccc"), Some("affected"), Some("It is called."), Some("Vega parses input.")).await.unwrap_err();
    assert!(refused.message().contains("GHSA-cccc-cccc-cccc does not reach `project:vega`") && refused.message().contains("GHSA-bbbb-bbbb-bbbb"), "{}", refused.message());

    let first = world.services.exposures.assess("vega", Some("GHSA-bbbb-bbbb-bbbb"), Some("affected"), Some("`parse` runs on every request body."), Some("Vega parses every request.")).await.unwrap();
    assert!(!first.replaced);
    let second = world.services.exposures.assess("vega", Some("GHSA-bbbb-bbbb-bbbb"), Some("affected"), Some("Still reached."), Some("Vega still parses every request.")).await.unwrap();
    assert!(second.replaced);

    let read = world.services.exposures.read("vega", Some("GHSA-bbbb-bbbb-bbbb")).await.unwrap();
    assert!(read.asked_for_one && read.inventoried);
    assert_eq!(read.exposures[0].reasoning.as_deref(), Some("Still reached."));
    assert_eq!(read.exposures[0].conclusion.as_deref(), Some("Vega still parses every request."));
    assert!(read.exposures[0].open, "an affected exposure stays a warning until a task fixes it");
}
