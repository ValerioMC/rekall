//! `RekallEndToEndTest`: the "Done when" checks that gate a claim, run through a scripted
//! runner so no real command starts, against real git repositories.

mod support;

use std::sync::Arc;

use rekall_app::StartOptions;
use rekall_core::check::{CheckOutcome, ScriptedCheckRunner};
use serde_json::{json, Value};
use support::{a_git_repo_with_one_commit, app_with, run_git, uuid_value, App};

const STEP_ANCHORS: &str = "project:vega task:report-builder";

struct World {
    app: App,
    runner: Arc<ScriptedCheckRunner>,
    repo: tempfile::TempDir,
    project_id: String,
    task_id: String,
}

async fn world(runner: ScriptedCheckRunner) -> World {
    let runner = Arc::new(runner);
    let options = StartOptions { no_restart: true, check_runner: Some(runner.clone()), ..StartOptions::default() };
    let app = app_with(&[], options).await;
    let acme = app.a_company("Acme").await;
    let repo = a_git_repo_with_one_commit("seed");
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    app.put(
        &format!("/api/projects/{project_id}"),
        json!({ "label": "vega", "title": "Vega", "status": "ACTIVE", "companyId": acme, "repoFolder": repo.path().to_str().unwrap(), "autoCommit": true }),
    )
    .await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    World { app, runner, repo, project_id, task_id }
}

impl World {
    async fn add_check(&self, label: &str, command: &str) {
        let created = self
            .app
            .post(&format!("/api/projects/{}/checks", self.project_id), json!({ "label": label, "command": command, "timeoutSeconds": 60 }))
            .await;
        assert_eq!(created.status, 201, "{}", created.text);
    }

    async fn claim_step(&self) -> String {
        self.app.call_tool("rekall_step", json!({ "anchors": STEP_ANCHORS, "step": "1", "state": "claimed" })).await
    }

    async fn write_wrapup(&self) -> String {
        self.app.call_tool("rekall_wrapup", json!({ "anchors": STEP_ANCHORS, "body": "Lo stato corrente." })).await
    }

    async fn step_state(&self) -> Option<String> {
        self.app.string("SELECT state FROM task_step WHERE task_id = ?", vec![uuid_value(&self.task_id)]).await
    }

    async fn kept_results(&self) -> Vec<(bool, Option<i64>)> {
        self.app
            .query("SELECT passed, exit_code FROM done_check_result WHERE task_id = ? ORDER BY ran_at", vec![uuid_value(&self.task_id)])
            .await
            .iter()
            .map(|row| (row.try_get::<bool>("", "passed").unwrap(), row.try_get::<Option<i64>>("", "exit_code").unwrap()))
            .collect()
    }

    fn head_subject(&self) -> String {
        run_git(self.repo.path(), &["log", "-1", "--format=%s"])
    }
}

fn red(command: &str, output: &str) -> ScriptedCheckRunner {
    ScriptedCheckRunner::new().answering(command, CheckOutcome::Exited(1), output)
}

#[tokio::test]
async fn a_red_check_refuses_the_claim_with_its_output_and_leaves_the_step_and_the_tree_alone() {
    let world = world(red("cargo test", "test export::it_works ... FAILED")).await;
    world.app.a_step(&world.task_id, "Wire the export endpoint", None).await;
    world.add_check("backend tests", "cargo test").await;
    std::fs::write(world.repo.path().join("export.ts"), "export const x = 1\n").unwrap();

    let answer = world.claim_step().await;

    assert_contains!(answer, "Claim refused", "\"backend tests\" failed with exit code 1", "it_works ... FAILED", "The step is still `running`", "Do not change the check");
    assert_ne!(world.step_state().await.as_deref(), Some("CLAIMED"));
    assert_eq!(world.head_subject(), "seed", "nothing was committed");
    assert_contains!(run_git(world.repo.path(), &["status", "--porcelain"]), "export.ts");
    assert_eq!(world.kept_results().await, vec![(false, Some(1))], "the red run is kept for the console");
}

#[tokio::test]
async fn green_checks_let_the_claim_through_and_the_answer_names_them_before_the_commit() {
    let world = world(ScriptedCheckRunner::new()).await;
    world.app.a_step(&world.task_id, "Wire the export endpoint", None).await;
    world.add_check("backend tests", "cargo test").await;
    world.add_check("ui lint", "pnpm lint").await;
    std::fs::write(world.repo.path().join("export.ts"), "export const x = 1\n").unwrap();

    let answer = world.claim_step().await;

    assert_contains!(answer, "is now `claimed`", "Checks passed: backend tests, ui lint.", "Auto-committed");
    assert_eq!(world.head_subject(), "feat: Wire the export endpoint");
    assert_eq!(world.kept_results().await.len(), 2);
}

#[tokio::test]
async fn a_project_without_checks_claims_exactly_as_before() {
    let world = world(ScriptedCheckRunner::new()).await;
    world.app.a_step(&world.task_id, "Wire the export endpoint", None).await;

    let answer = world.claim_step().await;

    assert_contains!(answer, "is now `claimed`");
    assert_lacks!(answer, "Checks passed", "Claim refused");
    assert!(world.runner.runs().is_empty());
}

#[tokio::test]
async fn a_wrapup_that_claims_a_task_with_no_steps_is_refused_by_a_red_check_and_nothing_is_written() {
    let world = world(red("cargo test", "1 test failed")).await;
    world.add_check("backend tests", "cargo test").await;

    let answer = world.write_wrapup().await;

    assert_contains!(answer, "Claim refused", "\"backend tests\" failed", "The wrapup was not written.");
    assert_eq!(world.app.count("SELECT COUNT(*) FROM wrapup WHERE task_id = ?", vec![uuid_value(&world.task_id)]).await, 0);
    let task: Value = world.app.get_task(&world.task_id).await;
    assert_eq!(task["reviewState"], "OPEN");
}

#[tokio::test]
async fn a_wrapup_with_green_checks_claims_the_task_and_names_them() {
    let world = world(ScriptedCheckRunner::new()).await;
    world.add_check("backend tests", "cargo test").await;

    let answer = world.write_wrapup().await;

    assert_contains!(answer, "Wrapup written", "Checks passed: backend tests.");
    assert_eq!(world.app.get_task(&world.task_id).await["reviewState"], "CLAIMED");
}

#[tokio::test]
async fn no_tool_writes_or_edits_a_check() {
    let world = world(ScriptedCheckRunner::new()).await;

    let listed = world.app.rpc("tools/list", json!({})).await;
    let names: Vec<&str> = listed["result"]["tools"].as_array().unwrap().iter().filter_map(|tool| tool["name"].as_str()).collect();

    assert!(!names.iter().any(|name| name.contains("check")), "{names:?}");
}
