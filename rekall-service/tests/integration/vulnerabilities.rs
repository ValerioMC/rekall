//! The inventory a session writes, the scan that matches it against advisories through a fake
//! feed, the assessment a session gives an exposure, and what a scan is set to look for.

mod support;

use std::fs;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use rekall_common::Instant;
use rekall_model::{AdvisorySeverity, Ecosystem, ExposureVerdict, FixScope};
use rekall_service::connection::InMemorySecretStore;
use rekall_service::inventory::DeclaredDependency;
use rekall_service::vulnerability::{
    version_range, AdvisoryFeed, AdvisoryFeedFailure, AffectedPackage, PackageVersion, PublishedAdvisory, ScanScope, ScanSettingsService, VulnerabilityScanService,
};
use rekall_service::Tx;
use sea_orm::ConnectionTrait;
use support::{commit, git, init_with_identity, world, World};

/// What the feed answers one question with.
type Answer = Result<Vec<PublishedAdvisory>, AdvisoryFeedFailure>;

/// Answers every question about an ecosystem with the advisories the test set for it that reach one
/// of the packages asked about, the way GitHub's `affects` does, and remembers the questions.
#[derive(Default)]
struct FakeFeed {
    latest: Vec<PublishedAdvisory>,
    affecting: Mutex<Vec<(Ecosystem, Answer)>>,
    asked: Mutex<Vec<(Ecosystem, Vec<PackageVersion>, ScanScope)>>,
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

    async fn affecting(&self, ecosystem: Ecosystem, packages: &[PackageVersion], scope: ScanScope) -> Answer {
        self.asked.lock().unwrap().push((ecosystem, packages.to_vec(), scope));
        let affecting = self.affecting.lock().unwrap();
        let answer = affecting.iter().find(|(known, _)| *known == ecosystem).map(|(_, answer)| answer.clone()).unwrap_or(Ok(Vec::new()))?;
        Ok(answer.into_iter().filter(|advisory| reaches_any(advisory, packages)).collect())
    }
}

fn reaches_any(advisory: &PublishedAdvisory, packages: &[PackageVersion]) -> bool {
    advisory.packages.iter().any(|affected| {
        packages.iter().any(|asked| asked.name.eq_ignore_ascii_case(&affected.name) && version_range::contains(affected.vulnerable_range.as_deref().unwrap_or(">= 0"), &asked.version) == Some(true))
    })
}

fn advisory(ghsa_id: &str, ecosystem: Ecosystem, name: &str) -> PublishedAdvisory {
    PublishedAdvisory {
        ghsa_id: ghsa_id.into(),
        cve_id: Some("CVE-2026-0001".into()),
        summary: format!("Remote code execution in {name}"),
        description: "An attacker runs code.".into(),
        url: format!("https://github.com/advisories/{ghsa_id}"),
        published_at: Instant::now(),
        severity: AdvisorySeverity::Critical,
        malware: false,
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

fn line(ecosystem: Ecosystem, name: &str, range: &str, first_patched: &str) -> AffectedPackage {
    AffectedPackage { ecosystem, name: name.into(), vulnerable_range: Some(range.into()), first_patched: Some(first_patched.into()), vulnerable_functions: Vec::new() }
}

fn settings(world: &World) -> ScanSettingsService {
    ScanSettingsService::new(world.services.ctx.clone(), Arc::new(InMemorySecretStore::new()))
}

fn scanner(world: &World, feed: Arc<FakeFeed>) -> VulnerabilityScanService {
    scanner_with(world, feed, settings(world))
}

fn scanner_with(world: &World, feed: Arc<FakeFeed>, settings: ScanSettingsService) -> VulnerabilityScanService {
    let services = &world.services;
    VulnerabilityScanService::new(services.ctx.clone(), feed, services.inventory.clone(), services.advisories.clone(), services.exposures.clone(), settings)
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

#[tokio::test]
async fn each_version_of_a_package_is_asked_about_alone_and_carries_the_release_line_its_version_is_on() {
    let world = world().await;
    world.project("vega", None, false).await;
    world.services.inventory_recording.record("vega", &[declared("npm", "vitest", "2.1.9"), declared("npm", "vitest", "4.0.7"), declared("npm", "vitest", "4.1.2")]).await.unwrap();
    let mut two_lines = advisory("GHSA-5xrq-8626-4rwp", Ecosystem::Npm, "vitest");
    two_lines.packages = vec![line(Ecosystem::Npm, "vitest", ">= 4.0.0, < 4.1.0", "4.1.0"), line(Ecosystem::Npm, "vitest", "< 3.2.6", "3.2.6")];
    let feed = Arc::new(FakeFeed::answering(Ecosystem::Npm, Ok(vec![two_lines])));

    let report = scanner(&world, feed.clone()).scan().await.unwrap();

    assert_eq!(feed.asked.lock().unwrap().len(), 3, "one question per version of vitest");
    assert_eq!(report.new_exposures, 2);
    let mut found: Vec<(String, Option<String>, Option<String>)> =
        world.services.exposures.list().await.unwrap().into_iter().map(|exposure| (exposure.package_version, exposure.vulnerable_range, exposure.first_patched)).collect();
    found.sort();
    assert_eq!(
        found,
        vec![
            ("2.1.9".into(), Some("< 3.2.6".into()), Some("3.2.6".into())),
            ("4.0.7".into(), Some(">= 4.0.0, < 4.1.0".into()), Some("4.1.0".into())),
        ]
    );
}

#[tokio::test]
async fn an_assessment_covers_every_version_it_reaches_and_tells_how_far_the_code_moved_since() {
    let repo = tempfile::tempdir().unwrap();
    init_with_identity(repo.path());
    fs::write(repo.path().join("a.txt"), "a").unwrap();
    git(repo.path(), &["add", "-A"]);
    commit(repo.path(), "seed");
    let world = world().await;
    world.project("vega", Some(&repo.path().to_string_lossy()), false).await;
    world.services.inventory_recording.record("vega", &[declared("npm", "left-pad", "1.0.0"), declared("npm", "left-pad", "1.5.0")]).await.unwrap();
    scanner(&world, Arc::new(FakeFeed::answering(Ecosystem::Npm, Ok(vec![advisory("GHSA-bbbb-bbbb-bbbb", Ecosystem::Npm, "left-pad")])))).scan().await.unwrap();

    let written = world.services.exposures.assess("vega", Some("GHSA-bbbb-bbbb-bbbb"), Some("not_affected"), Some("`parse` is never called."), Some("Vega pads strings and parses none.")).await.unwrap();
    assert_eq!(written.versions, vec!["left-pad 1.0.0", "left-pad 1.5.0"]);
    let head = git(repo.path(), &["rev-parse", "HEAD"]);
    let judged = world.services.exposures.list().await.unwrap();
    assert!(judged.iter().all(|exposure| exposure.verdict == Some(ExposureVerdict::NotAffected) && exposure.assessed_commit.as_deref() == Some(head.as_str())));
    assert!(judged.iter().all(|exposure| exposure.commits_since_assessment == Some(0)));

    fs::write(repo.path().join("a.txt"), "b").unwrap();
    git(repo.path(), &["add", "-A"]);
    commit(repo.path(), "call parse");

    assert!(world.services.exposures.list().await.unwrap().iter().all(|exposure| exposure.commits_since_assessment == Some(1)));
    let read = world.services.exposures.read("vega", Some("GHSA-bbbb-bbbb-bbbb")).await.unwrap();
    assert_eq!(read.exposures.len(), 2);
    assert_eq!(read.exposures[0].commits_since_assessment, Some(1));
}

#[tokio::test]
async fn an_exposure_its_done_fix_task_left_in_place_warns_again_and_takes_a_new_fix() {
    let world = world().await;
    let vega = world.project("vega", None, false).await;
    world.services.inventory_recording.record("vega", &[declared("npm", "left-pad", "1.0.0")]).await.unwrap();
    scanner(&world, Arc::new(FakeFeed::answering(Ecosystem::Npm, Ok(vec![advisory("GHSA-bbbb-bbbb-bbbb", Ecosystem::Npm, "left-pad")])))).scan().await.unwrap();
    let exposure = world.services.exposures.list().await.unwrap().remove(0);
    let task = world.task(&vega, "fix-ghsa-bbbb-bbbb-bbbb").await;
    let mut tx = Tx::write(&world.services.ctx).await.unwrap();
    let linked = world.services.exposures.link_fix_in(&mut tx, exposure.id, task.id).await;
    tx.finish(linked).await.unwrap();
    assert!(!world.services.exposures.list().await.unwrap()[0].open, "a fix task at work takes it off the warnings");

    world.db().execute_unprepared(&format!("UPDATE task SET status = 'DONE' WHERE id = '{}'", task.id)).await.unwrap();

    let still_found = world.services.exposures.list().await.unwrap().remove(0);
    assert!(still_found.open && still_found.fix_task_done);
    let tx = Tx::read(&world.services.ctx).await.unwrap();
    let brief = world.services.exposures.fix_brief_in(&tx, exposure.id).await;
    let briefs = world.services.exposures.fix_briefs_of_project_in(&tx, vega.id, FixScope::Affected).await;
    tx.finish(Ok(())).await.unwrap();
    assert!(brief.is_ok(), "a done fix task does not block a new one");
    assert_eq!(briefs.unwrap().len(), 1);
}

#[tokio::test]
async fn a_scan_asks_for_what_its_scope_holds_and_a_saved_token_is_only_ever_told_as_saved() {
    let world = world().await;
    world.project("vega", None, false).await;
    world.services.inventory_recording.record("vega", &[declared("npm", "left-pad", "1.0.0")]).await.unwrap();
    let settings = settings(&world);
    assert_eq!(settings.read().await.unwrap().scope, ScanScope::default());

    let changed = settings.change_scope(Some(true), None).await.unwrap();
    let feed = Arc::new(FakeFeed::default());
    scanner_with(&world, feed.clone(), settings.clone()).scan().await.unwrap();

    assert_eq!(changed.scope, ScanScope { include_high: true, include_malware: false });
    assert_eq!(settings.change_scope(None, Some(true)).await.unwrap().scope, ScanScope { include_high: true, include_malware: true });
    assert_eq!(feed.asked.lock().unwrap()[0].2, ScanScope { include_high: true, include_malware: false });
    assert!(!changed.token_saved);
    assert!(settings.save_token(Some("ghp_ab cd")).await.unwrap_err().is_illegal_argument());
    assert!(settings.save_token(Some("  ghp_abcd  ")).await.unwrap().token_saved);
    assert!(!settings.forget_token().await.unwrap().token_saved);
}
