use std::sync::Arc;

use rekall_core::decision::{AskDecision, DecisionService};
use rekall_core::EventBus;
use rekall_store::{company, project, task, task_step, ProjectStatus, TaskStepState};
use sea_orm::{ActiveModelTrait, IntoActiveModel};

use super::*;

struct World {
    service: TaskDecisionService,
    sessions: DecisionService,
    task: task::Model,
    _database: rekall_store::Database,
}

async fn world_with_steps(titles: &[&str]) -> World {
    let database = rekall_store::open_temporary().await.unwrap();
    let mut ctx = Ctx::new(database.conn.clone(), EventBus::new());
    let now = Instant::parse("2026-11-23T12:00:00Z").unwrap();
    ctx.clock = Arc::new(move || now);
    let db = &database.conn;
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(db).await.unwrap();
    let project = project::Model {
        id: Id::random(), label: "vega".into(), title: "Vega".into(), status: ProjectStatus::Active, icon: "folder".into(),
        description: None, blueprint_markdown: None, repo_folder: None, auto_commit: false, main_branch: None, pull_request_flow: false, task_key: "VG".into(),
        company_id: company.id, created_at: now, updated_at: now,
    };
    project.clone().into_active_model().insert(db).await.unwrap();
    let mut task = task::Model::new("deploy".into(), "Deploy".into(), project.id, now);
    task.number = 1;
    task.clone().into_active_model().insert(db).await.unwrap();
    for (position, title) in (0..).zip(titles) {
        let mut step = task_step::Model::new(task.id, (*title).into(), position, now);
        step.state = TaskStepState::Open;
        step.into_active_model().insert(db).await.unwrap();
    }
    World { service: TaskDecisionService::new(ctx.clone()), sessions: DecisionService::new(ctx), task, _database: database }
}

fn adding(question: &str, answer: &str) -> AddDecision {
    AddDecision { topic: None, question: Some(question.into()), answer: Some(answer.into()), step_number: None }
}

fn editing(answer: Option<&str>, note: Option<&str>) -> EditDecision {
    EditDecision { answer: answer.map(str::to_string), note: note.map(str::to_string) }
}

async fn asked_by_a_session(world: &World, key: &str) -> DecisionView {
    let command = AskDecision {
        key: key.into(), question: Some("Which region?".into()), topic: None, why: None, step_number: None, options: Vec::new(),
        source: DecisionSource::Terminal, lines: Vec::new(),
    };
    world.sessions.ask(None, "deploy", command).await.unwrap()
}

#[tokio::test]
async fn a_decision_written_by_hand_is_answered_and_marked_as_the_persons() {
    let world = world_with_steps(&["Draw", "Wire"]).await;

    let view = world.service.add(world.task.id, AddDecision { topic: Some("Cluster".into()), step_number: Some(2), ..adding("Which namespace?", "billing") }).await.unwrap();

    assert_eq!((view.key.as_str(), view.state, view.source, view.answer.as_deref()), ("which-namespace", DecisionState::Answered, DecisionSource::Person, Some("billing")));
    assert_eq!((view.topic.as_deref(), view.step_number, view.step_title.as_deref()), (Some("Cluster"), Some(2), Some("Wire")));
}

#[tokio::test]
async fn two_questions_that_make_the_same_key_get_distinct_ones() {
    let world = world_with_steps(&[]).await;
    world.service.add(world.task.id, adding("Which namespace?", "billing")).await.unwrap();

    let second = world.service.add(world.task.id, adding("Which namespace?", "payments")).await.unwrap();

    assert_eq!(second.key, "which-namespace-2");
}

#[tokio::test]
async fn adding_without_a_question_or_answer_or_with_a_step_the_task_lacks_is_refused() {
    let world = world_with_steps(&[]).await;

    let no_question = world.service.add(world.task.id, AddDecision { question: None, ..adding("", "x") }).await.unwrap_err();
    let no_answer = world.service.add(world.task.id, AddDecision { answer: Some(" ".into()), ..adding("Q?", "") }).await.unwrap_err();
    let stepless = world.service.add(world.task.id, AddDecision { step_number: Some(1), ..adding("Q?", "a") }).await.unwrap_err();

    assert!(no_question.message().contains("needs a question"), "{no_question:?}");
    assert!(no_answer.message().contains("needs an answer"), "{no_answer:?}");
    assert!(stepless.message().contains("has no checklist"), "{stepless:?}");
    assert!(world.service.list_of(world.task.id).await.unwrap().decisions.is_empty());
}

#[tokio::test]
async fn editing_an_answer_keeps_the_replaced_one_adds_the_note_and_marks_the_record() {
    let world = world_with_steps(&[]).await;
    let added = world.service.add(world.task.id, adding("How many replicas?", "2")).await.unwrap();

    let edited = world.service.edit(added.id, editing(Some("3"), Some("Autoscale to 6."))).await.unwrap();

    assert_eq!((edited.answer.as_deref(), edited.previous_answer.as_deref(), edited.source), (Some("3"), Some("2"), DecisionSource::Person));
    assert_eq!(edited.details.iter().map(|detail| detail.text.as_str()).collect::<Vec<_>>(), ["Autoscale to 6."]);
}

#[tokio::test]
async fn saving_the_same_answer_again_keeps_what_it_replaced_untouched() {
    let world = world_with_steps(&[]).await;
    let added = world.service.add(world.task.id, adding("How many replicas?", "2")).await.unwrap();
    world.service.edit(added.id, editing(Some("3"), None)).await.unwrap();

    let again = world.service.edit(added.id, editing(Some("3"), Some("Confirmed."))).await.unwrap();

    assert_eq!((again.answer.as_deref(), again.previous_answer.as_deref()), (Some("3"), Some("2")));
}

#[tokio::test]
async fn a_person_answers_a_question_a_session_left_open_and_the_record_becomes_theirs() {
    let world = world_with_steps(&[]).await;
    let open = asked_by_a_session(&world, "region").await;

    let edited = world.service.edit(open.id, editing(Some("eu-west-1"), None)).await.unwrap();

    assert_eq!((edited.state, edited.answer.as_deref(), edited.previous_answer, edited.source), (DecisionState::Answered, Some("eu-west-1"), None, DecisionSource::Person));
}

#[tokio::test]
async fn an_edit_that_carries_nothing_is_refused_and_an_unanswered_decision_needs_its_answer() {
    let world = world_with_steps(&[]).await;
    let open = asked_by_a_session(&world, "region").await;
    let added = world.service.add(world.task.id, adding("How many replicas?", "2")).await.unwrap();

    let noteless_open = world.service.edit(open.id, editing(None, Some("Only a note"))).await.unwrap_err();
    let empty = world.service.edit(added.id, editing(None, None)).await.unwrap_err();

    assert!(noteless_open.message().contains("has no answer yet"), "{noteless_open:?}");
    assert!(empty.message().contains("a new answer, a note, or both"), "{empty:?}");
}

#[tokio::test]
async fn a_removed_decision_is_gone_and_an_unknown_one_is_not_found() {
    let world = world_with_steps(&[]).await;
    let added = world.service.add(world.task.id, adding("Which namespace?", "billing")).await.unwrap();

    world.service.remove(added.id).await.unwrap();
    let unknown = world.service.remove(added.id).await.unwrap_err();

    assert!(world.service.list_of(world.task.id).await.unwrap().decisions.is_empty());
    assert!(matches!(unknown, RekallError::NotFound(_)), "{unknown:?}");
}

#[tokio::test]
async fn the_list_carries_the_text_the_context_has_for_a_session_with_the_persons_mark() {
    let world = world_with_steps(&[]).await;
    world.service.add(world.task.id, adding("Which namespace?", "billing")).await.unwrap();

    let listed = world.service.list_of(world.task.id).await.unwrap();

    assert_eq!(listed.decisions.len(), 1);
    assert_eq!(listed.session_block, "<decisions count=\"1\" answered=\"1\" open=\"0\" on-hold=\"0\">\nwhich-namespace: billing [edited by you]\n</decisions>");
}

#[tokio::test]
async fn listing_the_decisions_of_an_unknown_task_is_not_found() {
    let world = world_with_steps(&[]).await;

    let unknown = world.service.list_of(Id::random()).await.unwrap_err();

    assert!(matches!(unknown, RekallError::NotFound(_)), "{unknown:?}");
}
