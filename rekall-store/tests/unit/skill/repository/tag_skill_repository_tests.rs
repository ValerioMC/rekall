use super::*;
use crate::tag;
use rekall_kernel::Instant;
use sea_orm::{ActiveModelTrait, IntoActiveModel};

async fn a_tag(db: &impl ConnectionTrait) -> tag::Model {
    let now = Instant::now();
    let tag = tag::Model { id: Id::random(), name: "frontend".into(), icon: "tag".into(), color: "blue".into(), created_at: now, updated_at: now };
    tag.clone().into_active_model().insert(db).await.unwrap();
    tag
}

fn skill(tag: &tag::Model, name: &str, arguments: Option<&str>, position: i32) -> Model {
    Model { id: Id::random(), tag_id: tag.id, skill_name: name.into(), arguments: arguments.map(str::to_string), position, created_at: Instant::now() }
}

#[tokio::test]
async fn the_arguments_of_a_tag_skill_survive_a_round_trip() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let tag = a_tag(db).await;

    replace(db, tag.id, vec![skill(&tag, "impeccable", Some("website ui"), 0), skill(&tag, "rk", None, 1)]).await.unwrap();

    let stored: Vec<(String, Option<String>)> = list_of_tag(db, tag.id).await.unwrap().into_iter().map(|row| (row.skill_name, row.arguments)).collect();
    assert_eq!(stored, [("impeccable".to_string(), Some("website ui".to_string())), ("rk".to_string(), None)]);
}
