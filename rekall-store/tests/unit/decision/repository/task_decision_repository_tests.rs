use super::*;
use crate::decision::enums::{DecisionSource, Speaker};
use crate::enums::ProjectStatus;
use crate::{company, project, task};
use rekall_kernel::Instant;
use sea_orm::{ActiveModelTrait, IntoActiveModel};

async fn a_task(db: &impl ConnectionTrait) -> task::Model {
    let now = Instant::now();
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(db).await.unwrap();
    let project = project::Model {
        id: Id::random(), label: "vega".into(), title: "Vega".into(), status: ProjectStatus::Active, icon: "folder".into(),
        description: None, blueprint_markdown: None, repo_folder: None, auto_commit: false, main_branch: None, pull_request_flow: false, task_key: String::new(), company_id: company.id,
        created_at: now, updated_at: now,
    };
    project.clone().into_active_model().insert(db).await.unwrap();
    let task = task::Model::new("report".into(), "Report".into(), project.id, now);
    task.clone().into_active_model().insert(db).await.unwrap();
    task
}

fn a_record(task: &task::Model, key: &str, step_number: Option<i32>) -> DecisionRecord {
    let now = Instant::now();
    let mut decision = Model::new(task.id, key.into(), "Where does the password live?".into(), DecisionSource::Terminal, now);
    decision.step_number = step_number;
    DecisionRecord {
        options: vec![task_decision_option::Model::new(decision.id, 2, "Vault".into()), task_decision_option::Model::new(decision.id, 1, "Secret".into())],
        lines: vec![task_decision_line::Model::new(decision.id, 1, Speaker::Claude, "Which store?".into(), now)],
        details: vec![task_decision_detail::Model::new(decision.id, "Rotate it every 90 days.".into(), now)],
        decision,
    }
}

#[tokio::test]
async fn a_decision_comes_back_with_its_options_lines_and_details() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let task = a_task(db).await;
    let record = a_record(&task, "db-password", None);
    insert(db, &record).await.unwrap();

    let listed = list_of_task(db, task.id).await.unwrap();

    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].decision, record.decision);
    assert_eq!(listed[0].options.iter().map(|option| option.label.as_str()).collect::<Vec<_>>(), ["Secret", "Vault"]);
    assert_eq!((listed[0].lines.len(), listed[0].details.len()), (1, 1));
}

#[tokio::test]
async fn the_tasks_own_decisions_come_before_each_steps() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let task = a_task(db).await;
    insert(db, &a_record(&task, "second-step", Some(2))).await.unwrap();
    insert(db, &a_record(&task, "first-step", Some(1))).await.unwrap();
    insert(db, &a_record(&task, "whole-task", None)).await.unwrap();

    let keys: Vec<String> = list_of_task(db, task.id).await.unwrap().into_iter().map(|record| record.decision.key).collect();

    assert_eq!(keys, ["whole-task", "first-step", "second-step"]);
}

#[tokio::test]
async fn a_second_decision_with_the_same_key_on_one_task_is_refused() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let task = a_task(db).await;
    insert(db, &a_record(&task, "db-password", None)).await.unwrap();

    let refused = insert(db, &a_record(&task, "db-password", Some(1))).await.unwrap_err();

    assert!(refused.message().contains("task_decision.task_id, task_decision.key"), "{refused:?}");
}

#[tokio::test]
async fn requiring_a_key_the_task_does_not_carry_is_not_found() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let task = a_task(db).await;

    let refused = require(db, task.id, "db-password").await.unwrap_err();

    assert!(matches!(refused, RekallError::NotFound(_)) && refused.message().contains("db-password"), "{refused:?}");
}

#[tokio::test]
async fn replacing_the_children_swaps_them_and_leaves_the_decision() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let task = a_task(db).await;
    let mut record = a_record(&task, "db-password", None);
    insert(db, &record).await.unwrap();
    record.options = vec![task_decision_option::Model::new(record.decision.id, 1, "Sealed secret".into())];
    record.details.clear();

    replace_children(db, &record).await.unwrap();

    let listed = list_of_task(db, task.id).await.unwrap();
    assert_eq!(listed[0].options, record.options);
    assert_eq!((listed[0].lines.len(), listed[0].details.len()), (1, 0));
    assert_eq!(listed[0].decision, record.decision);
}

#[tokio::test]
async fn deleting_a_task_deletes_its_decisions_and_what_hangs_from_them() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let task = a_task(db).await;
    insert(db, &a_record(&task, "db-password", None)).await.unwrap();

    task::Entity::delete_by_id(task.id).exec(db).await.unwrap();

    assert_eq!(Entity::find().all(db).await.unwrap().len(), 0);
    assert_eq!(task_decision_option::Entity::find().all(db).await.unwrap().len(), 0);
    assert_eq!(task_decision_line::Entity::find().all(db).await.unwrap().len(), 0);
    assert_eq!(task_decision_detail::Entity::find().all(db).await.unwrap().len(), 0);
}

#[tokio::test]
async fn deleting_one_decision_leaves_the_others() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let task = a_task(db).await;
    let doomed = a_record(&task, "db-password", None);
    insert(db, &doomed).await.unwrap();
    insert(db, &a_record(&task, "region", None)).await.unwrap();

    delete(db, doomed.decision.id).await.unwrap();

    let keys: Vec<String> = list_of_task(db, task.id).await.unwrap().into_iter().map(|record| record.decision.key).collect();
    assert_eq!(keys, ["region"]);
    assert_eq!(task_decision_option::Entity::find().all(db).await.unwrap().len(), 2);
}

#[tokio::test]
async fn a_state_source_or_speaker_outside_its_enum_is_refused_by_the_schema() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let task = a_task(db).await;
    let record = a_record(&task, "db-password", None);
    insert(db, &record).await.unwrap();
    let id = record.decision.id;

    let bad_state = format!("UPDATE task_decision SET state = 'MAYBE' WHERE id = '{id}'");
    let bad_source = format!("UPDATE task_decision SET source = 'EMAIL' WHERE id = '{id}'");
    let bad_speaker = format!("UPDATE task_decision_line SET speaker = 'ROBOT' WHERE decision_id = '{id}'");

    for statement in [bad_state, bad_source, bad_speaker] {
        let refused = db.execute_unprepared(&statement).await.unwrap_err().to_string();
        assert!(refused.contains("CHECK constraint failed"), "{statement}: {refused}");
    }
}

#[tokio::test]
async fn one_record_is_read_by_its_key_with_what_hangs_from_it() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let task = a_task(db).await;
    insert(db, &a_record(&task, "region", None)).await.unwrap();
    let record = a_record(&task, "db-password", Some(1));
    insert(db, &record).await.unwrap();

    let found = require_record(db, task.id, "db-password").await.unwrap();

    assert_eq!((found.decision, found.options.len(), found.lines, found.details), (record.decision, 2, record.lines, record.details));
}
