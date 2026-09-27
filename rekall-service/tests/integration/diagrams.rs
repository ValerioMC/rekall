//! Diagrams through the service: written whole, read back, replaced in place, generated for a
//! task with their spans held against the project folder, traced from code back to their
//! elements, and gone with their project.

mod support;

use rekall_common::RekallError;
use rekall_model::prelude::*;
use rekall_service::diagram::{DiagramDraft, DiagramService, GeneratedDiagram};
use rekall_service::DomainEvent;
use sea_orm::EntityTrait;
use serde_json::json;
use support::world;

fn order_flow() -> serde_json::Value {
    json!({
        "format": "rekall.semantic-graph",
        "version": 1,
        "nodes": [
            {"id": "process", "kind": "code", "title": "process_order()", "sources": [{"file": "src/order.rs", "startLine": 10, "endLine": 120, "symbol": "process_order"}]},
            {"id": "validate", "kind": "action", "title": "Validate order", "sources": [{"file": "src/order.rs", "startLine": 12, "endLine": 30}], "provenance": "inferred", "confidence": 0.9},
            {"id": "paid", "kind": "decision", "title": "Payment accepted?"}
        ],
        "edges": [
            {"from": "process", "to": "validate", "relation": "contains"},
            {"from": "validate", "to": "paid", "relation": "leads_to"}
        ]
    })
}

fn draft(project_id: rekall_common::Id, graph: serde_json::Value) -> DiagramDraft {
    DiagramDraft {
        id: None,
        project_id,
        task_id: None,
        title: " Order flow ".into(),
        question: "How is an order processed?".into(),
        graph: DiagramService::read_graph(graph).unwrap(),
    }
}

#[tokio::test]
async fn a_diagram_is_written_whole_listed_without_its_graph_and_read_back_in_full() {
    let world = world().await;
    let project = world.project("shop", None, false).await;
    let mut events = world.services.ctx.events.subscribe();

    let written = world.services.diagrams.write(draft(project.id, order_flow())).await.unwrap();

    assert_eq!(written.summary.title, "Order flow");
    assert_eq!((written.summary.node_count, written.summary.edge_count), (3, 2));
    let listed = world.services.diagrams.list().await.unwrap();
    assert_eq!(listed, vec![written.summary.clone()]);
    let read = world.services.diagrams.get(written.summary.id).await.unwrap();
    assert_eq!(read.graph, written.graph);
    assert_eq!(read.graph.edges[1].id, "e2");
    assert!(matches!(events.try_recv().unwrap(), DomainEvent::Diagram(event) if event.diagram_id == written.summary.id && !event.deleted));
}

#[tokio::test]
async fn replacing_keeps_the_id_and_creation_time_and_the_project() {
    let world = world().await;
    let project = world.project("shop", None, false).await;
    let other = world.project("elsewhere", None, false).await;
    let first = world.services.diagrams.write(draft(project.id, order_flow())).await.unwrap();

    let mut again = draft(project.id, order_flow());
    again.id = Some(first.summary.id);
    again.title = "Order flow, revised".into();
    let replaced = world.services.diagrams.write(again).await.unwrap();
    assert_eq!(replaced.summary.id, first.summary.id);
    assert_eq!(replaced.summary.created_at, first.summary.created_at);
    assert_eq!(world.services.diagrams.list().await.unwrap().len(), 1);

    let mut moved = draft(other.id, order_flow());
    moved.id = Some(first.summary.id);
    assert!(matches!(world.services.diagrams.write(moved).await, Err(RekallError::IllegalArgument(_))));
}

#[tokio::test]
async fn the_originating_task_has_to_be_on_the_same_project() {
    let world = world().await;
    let project = world.project("shop", None, false).await;
    let other = world.project("elsewhere", None, false).await;
    let foreign_task = world.task(&other, "billing").await;

    let mut asked = draft(project.id, order_flow());
    asked.task_id = Some(foreign_task.id);
    assert!(matches!(world.services.diagrams.write(asked).await, Err(RekallError::IllegalArgument(_))));
}

#[tokio::test]
async fn a_graph_breaking_the_rules_is_refused_with_every_reason() {
    let mut broken = order_flow();
    broken["edges"][1]["to"] = json!("ghost");
    broken["nodes"][2]["confidence"] = json!(3);

    let refused = DiagramService::read_graph(broken).unwrap_err();
    assert!(refused.message().contains("nodes[2].confidence"), "{refused}");
    assert!(refused.message().contains("edges[1].to"), "{refused}");

    let foreign = json!({"format": "mermaid", "version": 1, "nodes": []});
    assert!(DiagramService::read_graph(foreign).unwrap_err().message().contains("Unknown format"));
}

#[tokio::test]
async fn a_line_of_code_traces_back_to_the_elements_that_cover_it() {
    let world = world().await;
    let project = world.project("shop", None, false).await;
    let written = world.services.diagrams.write(draft(project.id, order_flow())).await.unwrap();

    let traces = world.services.diagrams.trace(project.id, "./src/order.rs", 20).await.unwrap();
    assert_eq!(traces.len(), 1);
    assert_eq!(traces[0].diagram_id, written.summary.id);
    let ids: Vec<&str> = traces[0].hits.iter().map(|hit| hit.node_id.as_str()).collect();
    assert_eq!(ids, vec!["validate", "process"]);

    assert!(world.services.diagrams.trace(project.id, "src/order.rs", 500).await.unwrap().is_empty());
}

#[tokio::test]
async fn a_deleted_task_leaves_the_diagram_and_a_deleted_project_takes_it() {
    let world = world().await;
    let project = world.project("shop", None, false).await;
    let task = world.task(&project, "orders").await;
    let mut asked = draft(project.id, order_flow());
    asked.task_id = Some(task.id);
    let written = world.services.diagrams.write(asked).await.unwrap();

    Task::delete_by_id(task.id).exec(world.db()).await.unwrap();
    assert_eq!(world.services.diagrams.get(written.summary.id).await.unwrap().summary.task_id, None);

    Project::delete_by_id(project.id).exec(world.db()).await.unwrap();
    assert!(world.services.diagrams.list().await.unwrap().is_empty());
}

/// A project folder holding `src/order.rs`, long enough for every span `order_flow` cites.
fn order_folder() -> tempfile::TempDir {
    let folder = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(folder.path().join("src")).unwrap();
    let body: String = (1..=130).map(|n| format!("line {n}\n")).collect();
    std::fs::write(folder.path().join("src/order.rs"), body).unwrap();
    folder
}

fn generated(graph: serde_json::Value) -> GeneratedDiagram {
    GeneratedDiagram {
        project_label: Some("shop".into()),
        task_label: "orders".into(),
        diagram_id: None,
        title: "Order flow".into(),
        question: "How is an order processed?".into(),
        graph: DiagramService::read_graph(graph).unwrap(),
    }
}

#[tokio::test]
async fn a_generated_diagram_lands_on_the_anchored_task_and_its_project() {
    let world = world().await;
    let folder = order_folder();
    let project = world.project("shop", Some(&folder.path().to_string_lossy()), false).await;
    let task = world.task(&project, "orders").await;

    let written = world.services.diagrams.write_generated(generated(order_flow())).await.unwrap();

    assert_eq!(written.diagram.summary.project_id, project.id);
    assert_eq!(written.diagram.summary.task_id, Some(task.id));
    assert_eq!(written.task_anchor, "project:shop task:orders");
    assert!(written.sources_checked && !written.replaced);

    let mut again = generated(order_flow());
    again.diagram_id = Some(written.diagram.summary.id);
    let replaced = world.services.diagrams.write_generated(again).await.unwrap();
    assert!(replaced.replaced);
    assert_eq!(world.services.diagrams.list().await.unwrap().len(), 1);
}

#[tokio::test]
async fn a_generated_diagram_citing_code_that_is_not_there_is_refused_and_not_stored() {
    let world = world().await;
    let folder = order_folder();
    let project = world.project("shop", Some(&folder.path().to_string_lossy()), false).await;
    world.task(&project, "orders").await;
    let mut guessed = order_flow();
    guessed["nodes"][0]["sources"][0]["endLine"] = json!(400);
    guessed["nodes"][1]["sources"][0]["file"] = json!("src/checkout.rs");

    let refused = world.services.diagrams.write_generated(generated(guessed)).await.unwrap_err();

    assert!(matches!(refused, RekallError::IllegalArgument(_)));
    assert!(refused.message().contains("nodes[0].sources[0].endLine"), "{refused}");
    assert!(refused.message().contains("nodes[1].sources[0].file"), "{refused}");
    assert!(world.services.diagrams.list().await.unwrap().is_empty());
}

#[tokio::test]
async fn without_a_project_folder_a_generated_diagram_is_stored_unchecked() {
    let world = world().await;
    let project = world.project("shop", None, false).await;
    world.task(&project, "orders").await;

    let written = world.services.diagrams.write_generated(generated(order_flow())).await.unwrap();
    assert!(!written.sources_checked);

    let mut unknown = generated(order_flow());
    unknown.task_label = "billing".into();
    assert!(matches!(world.services.diagrams.write_generated(unknown).await, Err(RekallError::UnknownAnchor(_))));
}
