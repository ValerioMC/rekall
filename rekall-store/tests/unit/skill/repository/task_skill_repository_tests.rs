use super::*;
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

fn skill(task: &task::Model, name: &str, arguments: Option<&str>, position: i32) -> Model {
    Model { id: Id::random(), task_id: task.id, step_id: None, skill_name: name.into(), arguments: arguments.map(str::to_string), position, created_at: Instant::now() }
}

#[tokio::test]
async fn the_arguments_of_a_skill_survive_a_round_trip_and_a_skill_without_them_has_none() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let task = a_task(db).await;

    replace(db, task.id, None, vec![skill(&task, "impeccable", Some("redesign website ui"), 0), skill(&task, "rk", None, 1)]).await.unwrap();

    let stored: Vec<(String, Option<String>)> = list_of_task(db, task.id).await.unwrap().into_iter().map(|row| (row.skill_name, row.arguments)).collect();
    assert_eq!(stored, [("impeccable".to_string(), Some("redesign website ui".to_string())), ("rk".to_string(), None)]);
}

#[tokio::test]
async fn replacing_the_list_replaces_the_arguments_too() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let task = a_task(db).await;
    replace(db, task.id, None, vec![skill(&task, "impeccable", Some("redesign"), 0)]).await.unwrap();

    replace(db, task.id, None, vec![skill(&task, "impeccable", Some("polish"), 0)]).await.unwrap();

    let arguments: Vec<Option<String>> = list_of_task(db, task.id).await.unwrap().into_iter().map(|row| row.arguments).collect();
    assert_eq!(arguments, [Some("polish".to_string())]);
}
