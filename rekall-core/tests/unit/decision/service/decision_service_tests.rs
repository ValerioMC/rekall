use std::sync::Arc;

use rekall_kernel::Instant;
use rekall_store::{company, project, task, task_step, DecisionSource, DecisionState, ProjectStatus, Speaker, TaskStepState};
use sea_orm::{ActiveModelTrait, IntoActiveModel};

use super::*;
use crate::check::ScriptedCheckRunner;
use crate::{EventBus, Services};

const NOW: &str = "2026-11-23T12:00:00Z";

struct World {
    services: Services,
    task: task::Model,
    _database: rekall_store::Database,
}

async fn world_with_steps(titles: &[&str]) -> World {
    let database = rekall_store::open_temporary().await.unwrap();
    let mut ctx = Ctx::new(database.conn.clone(), EventBus::new());
    let now = Instant::parse(NOW).unwrap();
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
    World { services: Services::new(ctx, Arc::new(ScriptedCheckRunner::new())), task, _database: database }
}

fn asking(key: &str, question: &str) -> AskDecision {
    AskDecision { key: key.into(), question: Some(question.into()), topic: None, why: None, step_number: None, options: Vec::new(), source: DecisionSource::Terminal, lines: Vec::new() }
}

fn option(label: &str, recommended: bool) -> OfferedOption {
    OfferedOption { label: label.into(), effect: Some(format!("{label} costs something")), recommended }
}

fn answering(key: &str, answer: &str) -> AnswerDecision {
    AnswerDecision { key: key.into(), answer: Some(answer.into()), detail: None, lines: Vec::new() }
}

fn correcting(key: &str, answer: Option<&str>, detail: Option<&str>) -> CorrectDecision {
    CorrectDecision { key: key.into(), answer: answer.map(str::to_string), detail: detail.map(str::to_string) }
}

#[tokio::test]
async fn a_question_is_recorded_open_with_its_options_and_the_lines_it_came_from() {
    let world = world_with_steps(&["Draw", "Wire"]).await;
    let mut command = asking("db-password", "Where does the password live?");
    command.step_number = Some(2);
    command.options = vec![option("Secret", false), option("Vault", true)];
    command.lines = vec![SpokenLine { speaker: Speaker::Claude, text: "Which store?".into() }, SpokenLine { speaker: Speaker::Person, text: "Not sure yet".into() }];

    let view = world.services.decisions.ask(Some("vega"), "deploy", command).await.unwrap();

    assert_eq!((view.state, view.step_number, view.step_title.as_deref(), view.answer.as_deref()), (DecisionState::Open, Some(2), Some("Wire"), None));
    assert_eq!(view.options.iter().map(|o| (o.label.as_str(), o.recommended)).collect::<Vec<_>>(), [("Secret", false), ("Vault", true)]);
    assert_eq!(view.lines.iter().map(|line| line.speaker).collect::<Vec<_>>(), [Speaker::Claude, Speaker::Person]);
    assert_eq!(world.services.decisions.list(world.task.id).await.unwrap(), vec![view]);
}

#[tokio::test]
async fn a_key_the_task_already_carries_is_refused_with_the_way_out() {
    let world = world_with_steps(&[]).await;
    world.services.decisions.ask(None, "deploy", asking("region", "Which region?")).await.unwrap();

    let refused = world.services.decisions.ask(None, "deploy", asking("region", "Which region again?")).await.unwrap_err();

    assert!(refused.message().contains("Use correct"), "{refused:?}");
    assert_eq!(world.services.decisions.list(world.task.id).await.unwrap().len(), 1);
}

#[tokio::test]
async fn every_refusal_of_a_question_writes_nothing() {
    let world = world_with_steps(&["Draw"]).await;
    let mut wrong_step = asking("a", "Q?");
    wrong_step.step_number = Some(2);
    let mut seven_options = asking("b", "Q?");
    seven_options.options = (0..7).map(|n| option(&format!("o{n}"), false)).collect();
    let mut thirteen_lines = asking("c", "Q?");
    thirteen_lines.lines = (0..13).map(|n| SpokenLine { speaker: Speaker::Claude, text: format!("line {n}") }).collect();
    let mut long_why = asking("d", "Q?");
    long_why.why = Some("w".repeat(2_001));
    let refusals = [
        (wrong_step, "checklist has 1 step: there is no step 2"),
        (seven_options, "at most 6 options"),
        (thirteen_lines, "at most 12 conversation lines"),
        (long_why, "capped at 2000"),
        (asking("Not A Slug", "Q?"), "lowercase slug"),
        (AskDecision { question: Some("   ".into()), ..asking("e", "") }, "needs a question"),
        (AskDecision { question: Some("q".repeat(501)), ..asking("f", "") }, "capped at 500"),
    ];

    for (command, expected) in refusals {
        let refused = world.services.decisions.ask(None, "deploy", command).await.unwrap_err();
        assert!(refused.is_illegal_argument() && refused.message().contains(expected), "{expected}: {refused:?}");
    }
    assert!(world.services.decisions.list(world.task.id).await.unwrap().is_empty());
}

#[tokio::test]
async fn a_task_without_steps_accepts_no_step_number() {
    let world = world_with_steps(&[]).await;
    let mut command = asking("region", "Which region?");
    command.step_number = Some(1);

    let refused = world.services.decisions.ask(None, "deploy", command).await.unwrap_err();

    assert!(refused.message().contains("has no checklist"), "{refused:?}");
}

#[tokio::test]
async fn an_open_decision_is_answered_with_a_detail_and_the_lines_that_follow() {
    let world = world_with_steps(&[]).await;
    let mut command = asking("region", "Which region?");
    command.lines = vec![SpokenLine { speaker: Speaker::Claude, text: "Which region?".into() }];
    world.services.decisions.ask(None, "deploy", command).await.unwrap();

    let view = world
        .services
        .decisions
        .answer(None, "deploy", AnswerDecision { detail: Some("Only prod.".into()), lines: vec![SpokenLine { speaker: Speaker::Person, text: "eu-west-1".into() }], ..answering("region", "eu-west-1") })
        .await
        .unwrap();

    assert_eq!((view.state, view.answer.as_deref(), view.previous_answer), (DecisionState::Answered, Some("eu-west-1"), None));
    assert_eq!(view.details.iter().map(|d| d.text.as_str()).collect::<Vec<_>>(), ["Only prod."]);
    assert_eq!(view.lines.iter().map(|line| line.text.as_str()).collect::<Vec<_>>(), ["Which region?", "eu-west-1"]);
}

#[tokio::test]
async fn an_answer_that_is_empty_or_given_twice_is_refused() {
    let world = world_with_steps(&[]).await;
    world.services.decisions.ask(None, "deploy", asking("region", "Which region?")).await.unwrap();

    let empty = world.services.decisions.answer(None, "deploy", AnswerDecision { answer: Some(" ".into()), ..answering("region", "") }).await.unwrap_err();
    world.services.decisions.answer(None, "deploy", answering("region", "eu-west-1")).await.unwrap();
    let twice = world.services.decisions.answer(None, "deploy", answering("region", "us-east-1")).await.unwrap_err();

    assert!(empty.message().contains("needs answer"), "{empty:?}");
    assert!(twice.message().contains("Use correct"), "{twice:?}");
}

#[tokio::test]
async fn correcting_an_answer_keeps_the_old_one_and_when_it_changed() {
    let world = world_with_steps(&[]).await;
    world.services.decisions.ask(None, "deploy", asking("replicas", "How many replicas?")).await.unwrap();
    world.services.decisions.answer(None, "deploy", answering("replicas", "2")).await.unwrap();

    let view = world.services.decisions.correct(None, "deploy", correcting("replicas", Some("3"), Some("Autoscale to 6."))).await.unwrap();

    assert_eq!((view.answer.as_deref(), view.previous_answer.as_deref(), view.changed_at), (Some("3"), Some("2"), Some(Instant::parse(NOW).unwrap())));
    assert_eq!(view.details.len(), 1);
}

#[tokio::test]
async fn a_correction_may_only_add_a_detail_but_must_carry_something_and_needs_an_answer_to_correct() {
    let world = world_with_steps(&[]).await;
    world.services.decisions.ask(None, "deploy", asking("replicas", "How many replicas?")).await.unwrap();
    let unanswered = world.services.decisions.correct(None, "deploy", correcting("replicas", Some("3"), None)).await.unwrap_err();
    world.services.decisions.answer(None, "deploy", answering("replicas", "2")).await.unwrap();

    let empty = world.services.decisions.correct(None, "deploy", correcting("replicas", None, None)).await.unwrap_err();
    let detailed = world.services.decisions.correct(None, "deploy", correcting("replicas", None, Some("Per zone."))).await.unwrap();

    assert!(unanswered.message().contains("Use answer"), "{unanswered:?}");
    assert!(empty.message().contains("a new answer, a detail, or both"), "{empty:?}");
    assert_eq!((detailed.answer.as_deref(), detailed.previous_answer, detailed.details.len()), (Some("2"), None, 1));
}

#[tokio::test]
async fn a_decision_on_hold_carries_a_note_until_it_is_answered() {
    let world = world_with_steps(&[]).await;
    world.services.decisions.ask(None, "deploy", asking("vault-path", "Is there a Vault path?")).await.unwrap();

    let held = world.services.decisions.hold(None, "deploy", "vault-path", Some("Asking the\nplatform team")).await.unwrap();
    let answered = world.services.decisions.answer(None, "deploy", answering("vault-path", "secret/billing")).await.unwrap();

    assert_eq!((held.state, held.hold_note.as_deref(), held.answer), (DecisionState::OnHold, Some("Asking the platform team"), None));
    assert_eq!((answered.state, answered.hold_note, answered.answer.as_deref()), (DecisionState::Answered, None, Some("secret/billing")));
}

#[tokio::test]
async fn a_hold_needs_a_note_and_an_answered_decision_cannot_be_held() {
    let world = world_with_steps(&[]).await;
    world.services.decisions.ask(None, "deploy", asking("region", "Which region?")).await.unwrap();

    let noteless = world.services.decisions.hold(None, "deploy", "region", Some("  ")).await.unwrap_err();
    world.services.decisions.answer(None, "deploy", answering("region", "eu-west-1")).await.unwrap();
    let answered = world.services.decisions.hold(None, "deploy", "region", Some("Checking")).await.unwrap_err();

    assert!(noteless.message().contains("needs a note"), "{noteless:?}");
    assert!(answered.message().contains("use correct"), "{answered:?}");
}

#[tokio::test]
async fn a_removed_decision_is_gone_and_an_unknown_key_is_not_found() {
    let world = world_with_steps(&[]).await;
    world.services.decisions.ask(None, "deploy", asking("region", "Which region?")).await.unwrap();
    world.services.decisions.ask(None, "deploy", asking("helm", "Helm?")).await.unwrap();

    world.services.decisions.remove(None, "deploy", "region").await.unwrap();
    let unknown = world.services.decisions.remove(None, "deploy", "region").await.unwrap_err();

    assert_eq!(world.services.decisions.list(world.task.id).await.unwrap().iter().map(|view| view.key.as_str()).collect::<Vec<_>>(), ["helm"]);
    assert!(matches!(unknown, RekallError::NotFound(_)), "{unknown:?}");
}

#[tokio::test]
async fn the_context_carries_the_tasks_decisions_before_each_steps_under_its_step() {
    let world = world_with_steps(&["Draw", "Wire"]).await;
    let ask_for = |key: &str, step: Option<i32>| AskDecision { step_number: step, ..asking(key, "Q?") };
    world.services.decisions.ask(None, "deploy", ask_for("wire-key", Some(2))).await.unwrap();
    world.services.decisions.ask(None, "deploy", ask_for("whole-key", None)).await.unwrap();
    world.services.decisions.ask(None, "deploy", ask_for("draw-key", Some(1))).await.unwrap();
    world.services.decisions.answer(None, "deploy", answering("whole-key", "yes")).await.unwrap();

    let rendered = world.services.renderer.render(&world.services.context.for_task(world.task.id).await.unwrap());

    let at = |needle: &str| rendered.find(needle).unwrap_or_else(|| panic!("{needle} missing in {rendered}"));
    assert!(rendered.contains("<decisions count=\"3\" answered=\"1\" open=\"2\" on-hold=\"0\">"), "{rendered}");
    assert!(at("whole-key: yes") < at("<step n=\"1\" title=\"Draw\">") && at("<step n=\"1\" title=\"Draw\">") < at("draw-key") && at("draw-key") < at("<step n=\"2\" title=\"Wire\">") && at("<step n=\"2\" title=\"Wire\">") < at("wire-key"), "{rendered}");
}

#[tokio::test]
async fn the_decisions_block_is_the_text_the_context_carries() {
    let world = world_with_steps(&[]).await;
    assert_eq!(world.services.context.decisions_block(world.task.id).await.unwrap(), "");
    world.services.decisions.ask(None, "deploy", asking("region", "Which region?")).await.unwrap();

    let block = world.services.context.decisions_block(world.task.id).await.unwrap();

    assert_eq!(block, "<decisions count=\"1\" answered=\"0\" open=\"1\" on-hold=\"0\">\nregion: ? Which region?\n</decisions>");
}
