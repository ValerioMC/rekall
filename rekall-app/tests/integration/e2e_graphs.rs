//! A project's deployment graphs: written whole from the console, each step reaching a connection
//! the project binds, and laid on a task as a flow a session then reads with its connections.

mod support;

use serde_json::{json, Value};
use support::{app, id, App};

const ANCHORS: &str = "project:vega task:release-1";

struct World {
    app: App,
    project_id: String,
    gitlab: String,
    cluster: String,
    cluster_binding: String,
}

/// A project binding a GitLab instance and a cluster.
async fn world() -> World {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let gitlab = id(&app.post(&format!("/api/companies/{acme}/connections"), json!({ "kind": "GITLAB", "title": "GitLab", "baseUrl": "https://gitlab.example.com" })).await);
    let cluster = id(&app.post(&format!("/api/companies/{acme}/connections"), json!({ "kind": "KUBERNETES", "title": "Prod" })).await);
    app.post(&format!("/api/projects/{project_id}/connections"), json!({ "connectionId": gitlab, "target": "acme/vega" })).await;
    let cluster_binding = id(&app.post(&format!("/api/projects/{project_id}/connections"), json!({ "connectionId": cluster, "target": "apps" })).await);
    World { app, project_id, gitlab, cluster, cluster_binding }
}

/// Push, build, deploy, check: a failed build, deploy or check goes back to the push.
fn release(world: &World) -> Value {
    json!({
        "title": "Release",
        "description": "How vega reaches production.",
        "steps": [
            { "key": "push", "title": "Push commit", "detail": "Push the work.", "connectionId": world.gitlab, "maxVisits": 3,
              "routes": [{ "outcome": "pushed", "toKey": "build" }] },
            { "key": "build", "title": "Pipeline build", "connectionId": world.gitlab,
              "routes": [{ "outcome": "passed", "toKey": "deploy" }, { "outcome": "failed", "toKey": "push" }] },
            { "key": "deploy", "title": "Deploy", "connectionId": world.cluster,
              "routes": [{ "outcome": "deployed", "toKey": "check" }, { "outcome": "failed", "toKey": "push" }] },
            { "key": "check", "title": "Check", "connectionId": world.cluster,
              "routes": [{ "outcome": "healthy" }, { "outcome": "error", "toKey": "push" }] }
        ]
    })
}

fn titles(graph: &Value) -> Vec<String> {
    graph["steps"].as_array().unwrap().iter().map(|step| step["title"].as_str().unwrap().to_string()).collect()
}

#[tokio::test]
async fn a_graph_is_written_whole_listed_rewritten_and_deleted() {
    let world = world().await;
    let url = format!("/api/projects/{}/graphs", world.project_id);

    let created = world.app.post(&url, release(&world)).await;
    assert_eq!(created.status, 201, "{}", created.text);
    let graph = id(&created);
    let listed = world.app.get(&url).await.list();
    assert_eq!(titles(&listed[0]), ["Push commit", "Pipeline build", "Deploy", "Check"]);
    assert_eq!(listed[0]["problem"], Value::Null, "a loop with a way out can run");
    let push = &listed[0]["steps"][0];
    assert_eq!((push["connectionId"].as_str(), push["maxVisits"].as_i64()), (Some(world.gitlab.as_str()), Some(3)));
    assert_eq!(listed[0]["steps"][3]["routes"][1]["toStepId"], push["id"], "a route back is kept by the step it leads to");

    let mut shorter = release(&world);
    shorter["title"] = json!("Hotfix");
    shorter["steps"] = json!([{ "key": "push", "title": "Push commit", "routes": [{ "outcome": "pushed" }] }]);
    let rewritten = world.app.put(&format!("/api/graphs/{graph}"), shorter).await;
    assert_eq!((rewritten.status, rewritten.str("title")), (200, "Hotfix".into()));
    assert_eq!(world.app.count("SELECT COUNT(*) FROM graph_step", vec![]).await, 1, "the old steps are gone");

    world.app.delete(&format!("/api/graphs/{graph}")).await;
    assert!(world.app.get(&url).await.list().is_empty());
    assert_eq!(world.app.count("SELECT COUNT(*) FROM graph_route", vec![]).await, 0);
}

#[tokio::test]
async fn a_step_keeps_where_it_was_dragged_until_it_is_handed_back_to_the_layout() {
    let world = world().await;
    let mut placed = release(&world);
    placed["steps"][2]["flowX"] = json!(480);
    placed["steps"][2]["flowY"] = json!(260);

    let graph = world.app.post(&format!("/api/projects/{}/graphs", world.project_id), placed.clone()).await;
    assert_eq!(graph.status, 201, "{}", graph.text);
    let deploy = &graph.get("steps")[2];
    assert_eq!((deploy["flowX"].as_i64(), deploy["flowY"].as_i64()), (Some(480), Some(260)));
    assert_eq!(graph.get("steps")[0]["flowX"], Value::Null, "an undragged step is left to the layout");

    placed["steps"][2]["flowX"] = Value::Null;
    placed["steps"][2]["flowY"] = Value::Null;
    let tidied = world.app.put(&format!("/api/graphs/{}", id(&graph)), placed).await;
    assert_eq!(tidied.get("steps")[2]["flowX"], Value::Null);
}

#[tokio::test]
async fn a_graph_that_cannot_run_is_kept_and_says_why() {
    let world = world().await;
    let unfinished = json!({ "title": "Draft", "steps": [
        { "key": "push", "title": "Push commit", "routes": [{ "outcome": "pushed" }] },
        { "key": "orphan", "title": "Orphan" }
    ] });

    let created = world.app.post(&format!("/api/projects/{}/graphs", world.project_id), unfinished).await;

    assert_eq!(created.status, 201, "{}", created.text);
    assert_contains!(created.str("problem"), "never reaches 'Orphan'");
}

#[tokio::test]
async fn a_graph_is_refused_what_no_flow_could_hold_or_the_project_does_not_bind() {
    let world = world().await;
    let url = format!("/api/projects/{}/graphs", world.project_id);
    let acme_other = world.app.a_company("Other").await;
    let foreign = id(&world.app.post(&format!("/api/companies/{acme_other}/connections"), json!({ "kind": "KUBERNETES", "title": "Theirs" })).await);

    let refusals = [
        json!({ "title": "Release", "steps": [] }),
        json!({ "title": "Release", "steps": [{ "key": "a", "title": "Push", "routes": [{ "outcome": "again", "toKey": "a" }] }] }),
        json!({ "title": "Release", "steps": [{ "key": "a", "title": "Push", "routes": [{ "outcome": "next", "toKey": "b" }] }] }),
        json!({ "title": "Release", "steps": [{ "key": "a", "title": "Deploy", "connectionId": foreign }] }),
        json!({ "title": " ", "steps": [{ "key": "a", "title": "Push" }] }),
    ];
    for body in refusals {
        let refused = world.app.post(&url, body.clone()).await;
        assert_eq!(refused.status, 400, "{body}: {}", refused.text);
    }
    world.app.post(&url, release(&world)).await;
    assert_eq!(world.app.post(&url, json!({ "title": "release", "steps": [{ "key": "a", "title": "Push" }] })).await.status, 409);
}

#[tokio::test]
async fn a_graph_laid_on_a_task_with_only_its_description_releases_a_generated_implementation_step() {
    let world = world().await;
    let task_id = world.app.a_task(&world.project_id, "release-1").await;
    let graph = id(&world.app.post(&format!("/api/projects/{}/graphs", world.project_id), release(&world)).await);

    let laid = world.app.post(&format!("/api/tasks/{task_id}/graph"), json!({ "graphId": graph })).await;

    assert_eq!(laid.status, 200, "{}", laid.text);
    let steps = laid.list();
    assert_eq!(steps.iter().map(|step| step["state"].as_str().unwrap()).collect::<Vec<_>>(), ["OPEN"; 5], "ready to work, not drafts");
    assert_eq!(steps[0]["title"], json!("Implement the task"));
    assert_eq!(steps[3]["connectionId"].as_str(), Some(world.cluster.as_str()));
    assert_eq!(steps[1]["maxVisits"].as_i64(), Some(3));
    let context = world.app.call_tool("rekall_context", json!({ "anchors": ANCHORS })).await;
    assert_contains!(
        context,
        "flow=\"true\"",
        "the flow begins at step 1 \"Implement the task\"",
        "on this step the description is the brief",
        "Routes: `done` -> step 2 \"Push commit\"",
        "- [ ] Push commit  (entered 0 of at most 3 times)\n  Connection: `gitlab`, reached as its line in `<connections>` says.\n  Push the work.",
        "- [ ] Deploy\n  Connection: `prod`",
        "Routes: `healthy` -> end of the flow; `error` -> step 2 \"Push commit\""
    );

    let again = world.app.post(&format!("/api/tasks/{task_id}/graph"), json!({ "graphId": graph })).await;
    assert_eq!(again.status, 409, "a graph is laid once: {}", again.text);
    assert_contains!(again.detail(), "already has a step named 'Push commit'");
}

#[tokio::test]
async fn the_step_a_failure_sends_the_work_to_reads_what_the_failing_step_reported() {
    let world = world().await;
    let task_id = world.app.a_task(&world.project_id, "release-1").await;
    let graph = id(&world.app.post(&format!("/api/projects/{}/graphs", world.project_id), release(&world)).await);
    world.app.post(&format!("/api/tasks/{task_id}/graph"), json!({ "graphId": graph })).await;
    for step in ["1", "2"] {
        world.app.call_tool("rekall_step", json!({ "anchors": ANCHORS, "step": step, "state": "claimed" })).await;
    }

    let answer = world.app.call_tool("rekall_step", json!({
        "anchors": ANCHORS, "step": "Pipeline build", "state": "claimed", "outcome": "failed",
        "report": "Job `test` failed: `version_matches` expects 1.2, got 1.3."
    })).await;

    assert_contains!(answer, "Routed by `failed` to step 2 \"Push commit\"");
    assert_contains!(
        world.app.call_tool("rekall_context", json!({ "anchors": ANCHORS })).await,
        "The flow came here from step 3 \"Pipeline build\", which ended `failed` and reported this.",
        "<arrived-from step=\"3\" outcome=\"failed\">\n  Job `test` failed: `version_matches` expects 1.2, got 1.3.\n  </arrived-from>"
    );
    let steps = world.app.get(&format!("/api/tasks/{task_id}/steps")).await.list();
    assert_eq!(steps[2]["visits"][0]["report"], json!("Job `test` failed: `version_matches` expects 1.2, got 1.3."), "the console reads it too");
}

#[tokio::test]
async fn a_graph_follows_a_checklist_which_leads_into_it_in_order() {
    let world = world().await;
    let task_id = world.app.a_task(&world.project_id, "release-1").await;
    world.app.a_step(&task_id, "Write the route", None).await;
    world.app.a_step(&task_id, "Test the route", None).await;
    let graph = id(&world.app.post(&format!("/api/projects/{}/graphs", world.project_id), release(&world)).await);

    let laid = world.app.post(&format!("/api/tasks/{task_id}/graph"), json!({ "graphId": graph })).await;

    assert_eq!(laid.status, 200, "{}", laid.text);
    assert_eq!(world.app.labels_of_steps(&task_id).await, ["Write the route", "Test the route", "Push commit", "Pipeline build", "Deploy", "Check"]);
    assert_contains!(
        world.app.call_tool("rekall_context", json!({ "anchors": ANCHORS })).await,
        "the flow begins at step 1 \"Write the route\"",
        "Routes: `done` -> step 2 \"Test the route\"",
        "Routes: `done` -> step 3 \"Push commit\""
    );
}

#[tokio::test]
async fn a_finished_checklist_is_released_from_the_graphs_first_step() {
    let world = world().await;
    let task_id = world.app.a_task(&world.project_id, "release-1").await;
    let written = world.app.a_step(&task_id, "Write the route", None).await;
    world.app.patch(&format!("/api/steps/{written}"), json!({ "done": true })).await;
    let graph = id(&world.app.post(&format!("/api/projects/{}/graphs", world.project_id), release(&world)).await);

    world.app.post(&format!("/api/tasks/{task_id}/graph"), json!({ "graphId": graph })).await;

    assert_contains!(world.app.call_tool("rekall_context", json!({ "anchors": ANCHORS })).await, "current=\"2\"", "the work stands on step 2 \"Push commit\"");
}

#[tokio::test]
async fn a_drawn_flow_goes_on_into_the_graph_where_it_would_end() {
    let world = world().await;
    let task_id = world.app.a_task(&world.project_id, "release-1").await;
    let write = world.app.a_step(&task_id, "Write the route", None).await;
    let review = world.app.a_step(&task_id, "Review the route", None).await;
    world.app.post(&format!("/api/steps/{write}/links"), json!({ "toStepId": review, "outcome": "written" })).await;
    world.app.post(&format!("/api/steps/{review}/links"), json!({ "endsFlow": true, "outcome": "approved" })).await;
    world.app.post(&format!("/api/steps/{review}/links"), json!({ "toStepId": write, "outcome": "rejected" })).await;
    let graph = id(&world.app.post(&format!("/api/projects/{}/graphs", world.project_id), release(&world)).await);

    world.app.post(&format!("/api/tasks/{task_id}/graph"), json!({ "graphId": graph })).await;

    let context = world.app.call_tool("rekall_context", json!({ "anchors": ANCHORS })).await;
    assert_contains!(context, "Routes: `approved` -> step 3 \"Push commit\"; `rejected` -> step 1 \"Write the route\"");
    assert_lacks!(context, "`done` -> step 2");
}

#[tokio::test]
async fn a_graph_waits_for_the_tasks_drafts_to_be_promoted_or_deleted() {
    let world = world().await;
    let task_id = world.app.a_task(&world.project_id, "release-1").await;
    world.app.a_draft_step(&task_id, "Write the route", None).await;
    let graph = id(&world.app.post(&format!("/api/projects/{}/graphs", world.project_id), release(&world)).await);

    let refused = world.app.post(&format!("/api/tasks/{task_id}/graph"), json!({ "graphId": graph })).await;

    assert_eq!(refused.status, 409);
    assert_contains!(refused.detail(), "1 draft steps");
    assert_eq!(world.app.labels_of_steps(&task_id).await, ["Write the route"], "a refusal lays nothing");
}

#[tokio::test]
async fn a_project_releases_with_at_most_one_of_its_own_graphs() {
    let world = world().await;
    let url = format!("/api/projects/{}/graphs", world.project_id);
    let release = id(&world.app.post(&url, release(&world)).await);
    let hotfix = id(&world.app.post(&url, json!({ "title": "Hotfix", "steps": [{ "key": "a", "title": "Push" }] })).await);
    let choose = format!("/api/projects/{}/release-graph", world.project_id);
    let defaults = |graphs: Vec<Value>| graphs.iter().map(|graph| graph["releaseDefault"].as_bool().unwrap()).collect::<Vec<_>>();

    assert_eq!(defaults(world.app.put(&choose, json!({ "graphId": release })).await.list()), [true, false]);
    assert_eq!(defaults(world.app.put(&choose, json!({ "graphId": hotfix })).await.list()), [false, true], "choosing one clears the other");
    assert_eq!(defaults(world.app.put(&choose, json!({ "graphId": null })).await.list()), [false, false]);

    let acme = world.app.a_company("Initech").await;
    let elsewhere = world.app.a_project(&acme, "orion", "ACTIVE").await;
    assert_eq!(world.app.put(&format!("/api/projects/{elsewhere}/release-graph"), json!({ "graphId": release })).await.status, 400);
}

#[tokio::test]
async fn running_a_task_lays_the_projects_release_graph_once() {
    let world = world().await;
    let task_id = world.app.a_task(&world.project_id, "release-1").await;
    world.app.a_step(&task_id, "Write the route", None).await;
    let graph = id(&world.app.post(&format!("/api/projects/{}/graphs", world.project_id), release(&world)).await);
    let run = format!("/api/tasks/{task_id}/release");

    let without_a_choice = world.app.post_empty(&run).await;
    assert_eq!(without_a_choice.body["laid"], Value::Null, "{}", without_a_choice.text);
    assert_eq!(without_a_choice.body["steps"].as_array().unwrap().len(), 1);

    world.app.put(&format!("/api/projects/{}/release-graph", world.project_id), json!({ "graphId": graph })).await;
    let laid = world.app.post_empty(&run).await;

    assert_eq!(laid.body["laid"], json!("Release"), "{}", laid.text);
    assert_eq!(world.app.labels_of_steps(&task_id).await, ["Write the route", "Push commit", "Pipeline build", "Deploy", "Check"]);
    assert!(world.app.get_task(&task_id).await["graphLaidAt"].is_string());
    assert_eq!(world.app.post_empty(&run).await.body["laid"], Value::Null, "a task is laid once");
}

#[tokio::test]
async fn a_task_left_unreleased_is_run_without_the_release_graph() {
    let world = world().await;
    let task_id = world.app.a_task(&world.project_id, "release-1").await;
    let graph = id(&world.app.post(&format!("/api/projects/{}/graphs", world.project_id), release(&world)).await);
    world.app.put(&format!("/api/projects/{}/release-graph", world.project_id), json!({ "graphId": graph })).await;

    let skipped = world.app.put(&format!("/api/tasks/{task_id}/release-skipped"), json!({ "skipped": true })).await;

    assert_eq!(skipped.body["releaseSkipped"], json!(true), "{}", skipped.text);
    assert_eq!(world.app.post_empty(&format!("/api/tasks/{task_id}/release")).await.body["laid"], Value::Null);
    assert!(world.app.labels_of_steps(&task_id).await.is_empty());
}

#[tokio::test]
async fn a_graph_laid_by_hand_counts_as_the_tasks_release() {
    let world = world().await;
    let task_id = world.app.a_task(&world.project_id, "release-1").await;
    let graph = id(&world.app.post(&format!("/api/projects/{}/graphs", world.project_id), release(&world)).await);
    world.app.put(&format!("/api/projects/{}/release-graph", world.project_id), json!({ "graphId": graph })).await;

    world.app.post(&format!("/api/tasks/{task_id}/graph"), json!({ "graphId": graph })).await;

    assert_eq!(world.app.post_empty(&format!("/api/tasks/{task_id}/release")).await.body["laid"], Value::Null);
}

#[tokio::test]
async fn a_graph_is_laid_only_on_its_own_projects_tasks_and_only_once_it_can_run() {
    let world = world().await;
    let acme = world.app.a_company("Initech").await;
    let elsewhere = world.app.a_project(&acme, "orion", "ACTIVE").await;
    let foreign_task = world.app.a_task(&elsewhere, "ship").await;
    let task_id = world.app.a_task(&world.project_id, "release-1").await;
    let url = format!("/api/projects/{}/graphs", world.project_id);
    let graph = id(&world.app.post(&url, release(&world)).await);
    let unfinished = id(&world.app.post(&url, json!({ "title": "Draft", "steps": [
        { "key": "push", "title": "Push commit", "routes": [{ "outcome": "pushed" }] },
        { "key": "orphan", "title": "Orphan" }
    ] })).await);

    assert_eq!(world.app.post(&format!("/api/tasks/{foreign_task}/graph"), json!({ "graphId": graph })).await.status, 400);
    let refused = world.app.post(&format!("/api/tasks/{task_id}/graph"), json!({ "graphId": unfinished })).await;
    assert_eq!(refused.status, 400);
    assert_contains!(refused.detail(), "cannot run yet");
    assert_eq!(world.app.count("SELECT COUNT(*) FROM task_step", vec![]).await, 0, "a refusal lays nothing");
}

#[tokio::test]
async fn a_connection_the_project_stops_using_is_reported_and_one_deleted_is_dropped() {
    let world = world().await;
    let task_id = world.app.a_task(&world.project_id, "release-1").await;
    let graph = id(&world.app.post(&format!("/api/projects/{}/graphs", world.project_id), release(&world)).await);
    world.app.post(&format!("/api/tasks/{task_id}/graph"), json!({ "graphId": graph })).await;

    world.app.delete(&format!("/api/projects/{}/connections/{}", world.project_id, world.cluster_binding)).await;
    assert_contains!(world.app.call_tool("rekall_context", json!({ "anchors": ANCHORS })).await, "`prod`, which this project no longer uses");

    world.app.delete(&format!("/api/connections/{}", world.gitlab)).await;
    let graphs = world.app.get(&format!("/api/projects/{}/graphs", world.project_id)).await.list();
    assert_eq!(graphs[0]["steps"][0]["connectionId"], Value::Null, "the graph's step reaches nothing");
    assert_lacks!(world.app.call_tool("rekall_context", json!({ "anchors": ANCHORS })).await, "`gitlab`");
}

#[tokio::test]
async fn deleting_a_project_takes_its_graphs_with_it() {
    let world = world().await;
    world.app.post(&format!("/api/projects/{}/graphs", world.project_id), release(&world)).await;

    world.app.delete(&format!("/api/projects/{}", world.project_id)).await;

    for table in ["deployment_graph", "graph_step", "graph_route"] {
        assert_eq!(world.app.count(&format!("SELECT COUNT(*) FROM {table}"), vec![]).await, 0, "{table}");
    }
}
