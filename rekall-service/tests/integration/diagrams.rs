//! Diagrams through the service: written whole, read back, replaced in place, generated for a
//! project or a task with their spans held against the project folder, and gone with their
//! project.

mod support;

use rekall_common::RekallError;
use rekall_diagram::DiagramKind;
use rekall_model::prelude::*;
use rekall_service::diagram::{DiagramDraft, DiagramService, DiagramTarget, GeneratedDiagram};
use rekall_service::DomainEvent;
use sea_orm::EntityTrait;
use serde_json::json;
use support::world;

const CHECKOUT: &str = "sequenceDiagram
  participant S as OrderService
  participant P as Payments
  S->>P: charge(card)
  P-->>S: receipt";

fn order_flow() -> serde_json::Value {
    json!({
        "sheets": [
            {
                "title": "Checkout",
                "mermaid": CHECKOUT,
                "links": {
                    "S": [{"file": "src/order.rs", "startLine": 10, "endLine": 120, "symbol": "OrderService"}],
                    "#1": [{"file": "src/order.rs", "startLine": 12, "endLine": 30, "symbol": "charge"}]
                }
            },
            { "mermaid": "stateDiagram-v2\n  [*] --> Pending\n  Pending --> Paid: pay" }
        ]
    })
}

fn draft(project_id: rekall_common::Id, document: serde_json::Value) -> DiagramDraft {
    DiagramDraft {
        id: None,
        project_id,
        task_id: None,
        title: " Order flow ".into(),
        question: "How is an order processed?".into(),
        document: DiagramService::read_document(document).unwrap(),
    }
}

#[tokio::test]
async fn a_diagram_is_written_whole_listed_with_its_kinds_and_read_back_in_full() {
    let world = world().await;
    let project = world.project("shop", None, false).await;
    let mut events = world.services.ctx.events.subscribe();

    let written = world.services.diagrams.write(draft(project.id, order_flow())).await.unwrap();

    assert_eq!(written.summary.title, "Order flow");
    assert_eq!(written.summary.kinds, vec![DiagramKind::Sequence, DiagramKind::State]);
    let listed = world.services.diagrams.list().await.unwrap();
    assert_eq!(listed, vec![written.summary.clone()]);
    let read = world.services.diagrams.get(written.summary.id).await.unwrap();
    assert_eq!(read, written);
    assert_eq!(read.sheets[0].kind, DiagramKind::Sequence);
    assert_eq!(read.sheets[0].links["#1"][0].symbol.as_deref(), Some("charge"));
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
async fn a_document_breaking_the_rules_is_refused_with_every_reason() {
    let mut broken = order_flow();
    broken["sheets"][0]["links"]["Ghost"] = json!([{"file": "src/order.rs"}]);
    broken["sheets"][1]["mermaid"] = json!("A --> B");

    let refused = DiagramService::read_document(broken).unwrap_err();
    assert!(refused.message().contains(r#"sheets[0].links["Ghost"]"#), "{refused}");
    assert!(refused.message().contains("sheets[1].mermaid"), "{refused}");

    let foreign = json!({"format": "rekall.semantic-graph", "nodes": []});
    assert!(DiagramService::read_document(foreign).unwrap_err().message().contains("Not a diagram document"));
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

fn on_task(document: serde_json::Value) -> GeneratedDiagram {
    GeneratedDiagram {
        target: DiagramTarget::Task { project_label: Some("shop".into()), task_label: "orders".into() },
        diagram_id: None,
        title: "Order flow".into(),
        question: "How is an order processed?".into(),
        document: DiagramService::read_document(document).unwrap(),
    }
}

#[tokio::test]
async fn a_generated_diagram_lands_on_the_anchored_task_and_its_project() {
    let world = world().await;
    let folder = order_folder();
    let project = world.project("shop", Some(&folder.path().to_string_lossy()), false).await;
    let task = world.task(&project, "orders").await;

    let written = world.services.diagrams.write_generated(on_task(order_flow())).await.unwrap();

    assert_eq!(written.diagram.summary.project_id, project.id);
    assert_eq!(written.diagram.summary.task_id, Some(task.id));
    assert_eq!(written.anchors, "project:shop task:orders");
    assert!(written.sources_checked && !written.replaced);

    let mut again = on_task(order_flow());
    again.diagram_id = Some(written.diagram.summary.id);
    let replaced = world.services.diagrams.write_generated(again).await.unwrap();
    assert!(replaced.replaced);
    assert_eq!(world.services.diagrams.list().await.unwrap().len(), 1);
}

#[tokio::test]
async fn a_generated_diagram_can_belong_to_the_project_alone() {
    let world = world().await;
    let project = world.project("shop", None, false).await;
    let mut generated = on_task(order_flow());
    generated.target = DiagramTarget::Project { label: "shop".into() };

    let written = world.services.diagrams.write_generated(generated).await.unwrap();

    assert_eq!(written.diagram.summary.project_id, project.id);
    assert_eq!(written.diagram.summary.task_id, None);
    assert_eq!(written.anchors, "project:shop");

    let mut unknown = on_task(order_flow());
    unknown.target = DiagramTarget::Project { label: "warehouse".into() };
    assert!(matches!(world.services.diagrams.write_generated(unknown).await, Err(RekallError::UnknownAnchor(_))));
}

#[tokio::test]
async fn a_generated_diagram_citing_code_that_is_not_there_is_refused_and_not_stored() {
    let world = world().await;
    let folder = order_folder();
    let project = world.project("shop", Some(&folder.path().to_string_lossy()), false).await;
    world.task(&project, "orders").await;
    let mut guessed = order_flow();
    guessed["sheets"][0]["links"]["S"][0]["endLine"] = json!(400);
    guessed["sheets"][0]["links"]["#1"][0]["file"] = json!("src/checkout.rs");

    let refused = world.services.diagrams.write_generated(on_task(guessed)).await.unwrap_err();

    assert!(matches!(refused, RekallError::IllegalArgument(_)));
    assert!(refused.message().contains(r#"sheets[0].links["S"][0].endLine"#), "{refused}");
    assert!(refused.message().contains(r##"sheets[0].links["#1"][0].file"##), "{refused}");
    assert!(world.services.diagrams.list().await.unwrap().is_empty());
}

#[tokio::test]
async fn without_a_project_folder_a_generated_diagram_is_stored_unchecked() {
    let world = world().await;
    let project = world.project("shop", None, false).await;
    world.task(&project, "orders").await;

    let written = world.services.diagrams.write_generated(on_task(order_flow())).await.unwrap();
    assert!(!written.sources_checked);

    let mut unknown = on_task(order_flow());
    unknown.target = DiagramTarget::Task { project_label: Some("shop".into()), task_label: "billing".into() };
    assert!(matches!(world.services.diagrams.write_generated(unknown).await, Err(RekallError::UnknownAnchor(_))));
}
