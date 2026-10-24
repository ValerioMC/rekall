//! A project's inventory written by a session, a scan against a fake advisory database, a
//! session's verdict, and the task the console opens to fix what remains.

mod support;

use std::sync::Arc;

use async_trait::async_trait;
use rekall_app::StartOptions;
use rekall_common::Instant;
use rekall_model::Ecosystem;
use rekall_service::vulnerability::{AdvisoryFeed, AdvisoryFeedFailure, AffectedPackage, PackageVersion, PublishedAdvisory};
use serde_json::json;
use support::{app_with, App};

/// Knows one critical advisory, on `lodash` below 4.17.21, and answers for it only when asked about a version it reaches.
struct LodashFeed;

fn lodash_advisory() -> PublishedAdvisory {
    PublishedAdvisory {
        ghsa_id: "GHSA-p6mc-m468-83gw".into(),
        cve_id: Some("CVE-2020-8203".into()),
        summary: "Prototype pollution in lodash".into(),
        description: "`zipObjectDeep` lets an attacker set properties on Object.prototype.".into(),
        url: "https://github.com/advisories/GHSA-p6mc-m468-83gw".into(),
        published_at: Instant::now(),
        packages: vec![AffectedPackage {
            ecosystem: Ecosystem::Npm,
            name: "lodash".into(),
            vulnerable_range: Some("< 4.17.21".into()),
            first_patched: Some("4.17.21".into()),
            vulnerable_functions: vec!["lodash.zipObjectDeep".into()],
        }],
    }
}

#[async_trait]
impl AdvisoryFeed for LodashFeed {
    async fn latest_critical(&self, _limit: u16) -> Result<Vec<PublishedAdvisory>, AdvisoryFeedFailure> {
        Ok(vec![lodash_advisory()])
    }

    async fn critical_affecting(&self, ecosystem: Ecosystem, packages: &[PackageVersion]) -> Result<Vec<PublishedAdvisory>, AdvisoryFeedFailure> {
        let reached = ecosystem == Ecosystem::Npm && packages.iter().any(|package| package.name == "lodash" && package.version != "4.17.21");
        Ok(if reached { vec![lodash_advisory()] } else { Vec::new() })
    }
}

/// Knows the lodash advisory and a second one on the same package, and answers with both for any lodash.
struct TwoLodashFeed;

fn command_injection_advisory() -> PublishedAdvisory {
    PublishedAdvisory {
        ghsa_id: "GHSA-35jh-r3h4-6jhm".into(),
        cve_id: Some("CVE-2021-23337".into()),
        summary: "Command injection in lodash".into(),
        description: "`template` runs code from its options.".into(),
        ..lodash_advisory()
    }
}

#[async_trait]
impl AdvisoryFeed for TwoLodashFeed {
    async fn latest_critical(&self, _limit: u16) -> Result<Vec<PublishedAdvisory>, AdvisoryFeedFailure> {
        Ok(vec![lodash_advisory(), command_injection_advisory()])
    }

    async fn critical_affecting(&self, ecosystem: Ecosystem, packages: &[PackageVersion]) -> Result<Vec<PublishedAdvisory>, AdvisoryFeedFailure> {
        let reached = ecosystem == Ecosystem::Npm && packages.iter().any(|package| package.name == "lodash");
        Ok(if reached { vec![lodash_advisory(), command_injection_advisory()] } else { Vec::new() })
    }
}

async fn vega() -> (App, String) {
    vega_with(Arc::new(LodashFeed)).await
}

async fn vega_with(feed: Arc<dyn AdvisoryFeed>) -> (App, String) {
    let app = app_with(&[], StartOptions { no_restart: true, advisory_feed: Some(feed), ..StartOptions::default() }).await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    (app, project_id)
}

async fn inventory(app: &App, lodash: &str) -> String {
    app.call_tool(
        "rekall_inventory",
        json!({
            "anchors": "project:vega",
            "dependencies": [
                { "ecosystem": "npm", "name": "lodash", "version": lodash, "manifest": "web/package-lock.json" },
                { "ecosystem": "rust", "name": "serde", "version": "1.0.200", "manifest": "Cargo.lock" }
            ]
        }),
    )
    .await
}

#[tokio::test]
async fn a_scan_raises_what_reaches_an_inventoried_project_and_a_new_inventory_settles_it() {
    let (app, project_id) = vega().await;
    let recorded = inventory(&app, "4.17.15").await;
    assert!(recorded.starts_with("Inventory of `project:vega` recorded: 2 dependencies (1 npm, 1 rust)."), "{recorded}");
    let listed = app.get(&format!("/api/projects/{project_id}/dependencies")).await.list();
    assert_eq!(listed.iter().map(|row| row["name"].as_str().unwrap()).collect::<Vec<_>>(), ["lodash", "serde"]);
    assert_eq!(listed[0]["ecosystem"], "NPM");
    let summaries = app.get("/api/inventories").await.list();
    assert_eq!((summaries.len(), summaries[0]["dependencyCount"].as_u64(), summaries[0]["ecosystems"]["RUST"].as_u64()), (1, Some(2), Some(1)));

    let report = app.post_empty("/api/vulnerabilities/scan").await;
    assert_eq!(report.status, 200, "{}", report.text);
    assert_eq!((report.get("newExposures").as_u64(), report.get("openExposures").as_u64(), report.get("projectsScanned").as_u64()), (Some(1), Some(1), Some(1)));

    let exposures = app.get("/api/vulnerabilities/exposures").await.list();
    assert_eq!(exposures.len(), 1);
    assert_eq!((exposures[0]["packageVersion"].as_str(), exposures[0]["firstPatched"].as_str(), exposures[0]["open"].as_bool()), (Some("4.17.15"), Some("4.17.21"), Some(true)));
    assert_eq!(app.get("/api/vulnerabilities/advisories").await.list()[0]["ghsaId"], "GHSA-p6mc-m468-83gw");

    let read = app.call_tool("rekall_vulnerabilities", json!({ "anchors": "project:vega", "advisory": "GHSA-p6mc-m468-83gw" })).await;
    assert!(read.contains("reaches `project:vega` through lodash 4.17.15 (npm)") && read.contains("Vulnerable functions: lodash.zipObjectDeep"), "{read}");

    let upgraded = inventory(&app, "4.17.21").await;
    assert!(upgraded.contains("One exposure whose package is gone at that version is no longer a warning."), "{upgraded}");
    assert!(app.get("/api/vulnerabilities/exposures").await.list().is_empty());
}

#[tokio::test]
async fn a_verdict_of_not_affected_takes_the_warning_off_and_affected_leaves_it_for_a_fix() {
    let (app, _) = vega().await;
    inventory(&app, "4.17.15").await;
    app.post_empty("/api/vulnerabilities/scan").await;

    let refused = app.call_tool("rekall_assess_vulnerability", json!({ "anchors": "project:vega", "advisory": "GHSA-xxxx-xxxx-xxxx", "verdict": "affected", "reasoning": "Called.", "conclusion": "Vega calls it." })).await;
    assert!(refused.contains("does not reach `project:vega`"), "{refused}");

    let judged = app
        .call_tool("rekall_assess_vulnerability", json!({ "anchors": "project:vega", "advisory": "GHSA-p6mc-m468-83gw", "verdict": "not_affected", "reasoning": "`zipObjectDeep` is never imported.", "conclusion": "Vega never builds objects from paths." }))
        .await;
    assert!(judged.contains("recorded as not affected. It is off the project's warnings."), "{judged}");
    let exposure = &app.get("/api/vulnerabilities/exposures").await.list()[0];
    assert_eq!((exposure["verdict"].as_str(), exposure["open"].as_bool()), (Some("NOT_AFFECTED"), Some(false)));
    assert_eq!(exposure["conclusion"].as_str(), Some("Vega never builds objects from paths."));

    let changed = app
        .call_tool("rekall_assess_vulnerability", json!({ "anchors": "project:vega", "advisory": "GHSA-p6mc-m468-83gw", "verdict": "affected", "reasoning": "`settings.ts` calls it on user input.", "conclusion": "Settings typed by a user reach it." }))
        .await;
    assert!(changed.contains("It replaces the earlier verdict.") && changed.contains("stays on the project's warnings"), "{changed}");
}

#[tokio::test]
async fn fixing_an_exposure_opens_a_briefed_task_once() {
    let (app, project_id) = vega().await;
    inventory(&app, "4.17.15").await;
    app.post_empty("/api/vulnerabilities/scan").await;
    let exposure_id = app.get("/api/vulnerabilities/exposures").await.list()[0]["id"].as_str().unwrap().to_string();
    app.a_task(&project_id, "fix-ghsa-p6mc-m468-83gw").await;

    let fixed = app.post_empty(&format!("/api/vulnerability-exposures/{exposure_id}/fix")).await;

    assert_eq!(fixed.status, 200, "{}", fixed.text);
    let task = fixed.get("task");
    assert_eq!(task["label"], "fix-ghsa-p6mc-m468-83gw-2", "a label the project uses is not taken over");
    assert_eq!(task["title"], "Fix GHSA-p6mc-m468-83gw in lodash 4.17.15");
    let description = task["description"].as_str().unwrap();
    assert!(description.contains("`lodash` 4.17.15 (npm, declared in `web/package-lock.json`)"), "{description}");
    assert!(description.contains("Upgrade `lodash` to 4.17.21 or later") && description.contains("`/rk project:vega inventory`"), "{description}");
    assert_eq!((fixed.get("exposure")["open"].as_bool(), fixed.get("exposure")["fixTaskId"].as_str()), (Some(false), task["id"].as_str()));

    let again = app.post_empty(&format!("/api/vulnerability-exposures/{exposure_id}/fix")).await;
    assert_eq!(again.status, 409, "{}", again.text);
}

#[tokio::test]
async fn fixing_a_project_opens_one_task_with_a_step_per_open_exposure_once() {
    let (app, project_id) = vega_with(Arc::new(TwoLodashFeed)).await;
    inventory(&app, "4.17.15").await;
    app.post_empty("/api/vulnerabilities/scan").await;

    let fixed = app.post_empty(&format!("/api/projects/{project_id}/vulnerabilities/fix")).await;

    assert_eq!(fixed.status, 200, "{}", fixed.text);
    let task = fixed.get("task");
    assert_eq!((task["label"].as_str(), task["title"].as_str()), (Some("fix-vulnerabilities"), Some("Fix 2 critical advisories in vega")));
    let steps = app.get(&format!("/api/tasks/{}/steps", task["id"].as_str().unwrap())).await.list();
    let mut titles: Vec<&str> = steps.iter().map(|step| step["title"].as_str().unwrap()).collect();
    titles.sort_unstable();
    assert_eq!(titles, ["Fix GHSA-35jh-r3h4-6jhm in lodash 4.17.15", "Fix GHSA-p6mc-m468-83gw in lodash 4.17.15"]);
    assert!(steps[0]["bodyMarkdown"].as_str().unwrap().contains("Upgrade `lodash` to 4.17.21 or later"), "{}", steps[0]);
    assert!(steps.iter().all(|step| step["state"] == "OPEN"), "the session the fix starts works them at once: {steps:?}");
    let exposures = fixed.get("exposures").as_array().unwrap().clone();
    assert!(exposures.iter().all(|exposure| exposure["fixTaskId"] == task["id"] && exposure["open"] == false), "{exposures:?}");

    let again = app.post_empty(&format!("/api/projects/{project_id}/vulnerabilities/fix")).await;
    assert_eq!(again.status, 409, "{}", again.text);
}

#[tokio::test]
async fn a_project_fix_of_the_affected_leaves_the_not_affected_and_a_fix_of_all_takes_them() {
    let (app, project_id) = vega_with(Arc::new(TwoLodashFeed)).await;
    inventory(&app, "4.17.15").await;
    app.post_empty("/api/vulnerabilities/scan").await;
    app.call_tool("rekall_assess_vulnerability", json!({ "anchors": "project:vega", "advisory": "GHSA-35jh-r3h4-6jhm", "verdict": "not_affected", "reasoning": "`template` is never called.", "conclusion": "Vega renders no templates." }))
        .await;
    let path = format!("/api/projects/{project_id}/vulnerabilities/fix");

    let affected = app.post(&path, json!({ "scope": "AFFECTED" })).await;
    assert_eq!(affected.status, 200, "{}", affected.text);
    let taken: Vec<&str> = affected.get("exposures").as_array().unwrap().iter().map(|exposure| exposure["advisory"]["ghsaId"].as_str().unwrap()).collect();
    assert_eq!(taken, ["GHSA-p6mc-m468-83gw"]);
    assert_eq!(app.post(&path, json!({ "scope": "AFFECTED" })).await.status, 409, "nothing is left a warning");

    let all = app.post(&path, json!({ "scope": "ALL" })).await;
    assert_eq!(all.status, 200, "{}", all.text);
    let taken: Vec<&str> = all.get("exposures").as_array().unwrap().iter().map(|exposure| exposure["advisory"]["ghsaId"].as_str().unwrap()).collect();
    assert_eq!(taken, ["GHSA-35jh-r3h4-6jhm"], "what the first fix took is not taken twice");
    assert!(all.get("task")["description"].as_str().unwrap().contains("(judged not affected)"), "{}", all.text);
    assert_eq!(app.post(&path, json!({ "scope": "ALL" })).await.status, 409);
}
