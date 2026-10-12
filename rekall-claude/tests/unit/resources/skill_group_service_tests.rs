use rekall_service::{Ctx, EventBus};

use super::*;

async fn service() -> (SkillGroupService, rekall_repository::Database) {
    let database = rekall_repository::open_temporary().await.unwrap();
    (SkillGroupService::new(Ctx::new(database.conn.clone(), EventBus::new())), database)
}

fn groups(views: &[SkillPlacementView]) -> Vec<(&str, &str)> {
    views.iter().map(|view| (view.group.as_str(), view.skill_path.as_str())).collect()
}

#[tokio::test]
async fn a_skill_is_shown_under_the_group_it_was_placed_in() {
    let (service, _database) = service().await;
    service.place("skills/python-standards.md", Some("Languages")).await.unwrap();
    let placed = service.place("skills/java-standards.md", Some(" Languages ")).await.unwrap();
    assert_eq!(groups(&placed), [("Languages", "skills/java-standards.md"), ("Languages", "skills/python-standards.md")]);
}

#[tokio::test]
async fn placing_a_skill_again_moves_it_and_naming_no_group_takes_it_out() {
    let (service, _database) = service().await;
    service.place("skills/docx/SKILL.md", Some("Office")).await.unwrap();
    let moved = service.place("skills/docx/SKILL.md", Some("Documents")).await.unwrap();
    assert_eq!(groups(&moved), [("Documents", "skills/docx/SKILL.md")]);
    let removed = service.place("skills/docx/SKILL.md", None).await.unwrap();
    assert!(removed.is_empty());
}

#[tokio::test]
async fn only_the_persons_own_skills_can_be_placed() {
    let (service, _database) = service().await;
    for refused in ["skills/synced/bucket/pdf/SKILL.md", "plugins/cache/m/p/1/skills/s/SKILL.md", "commands/rk.md", "../etc/passwd"] {
        let failure = service.place(refused, Some("Mine")).await.unwrap_err();
        assert!(matches!(failure, RekallError::IllegalArgument(_) | RekallError::Conflict(_)), "{refused}: {failure:?}");
    }
    assert!(service.placements().await.unwrap().is_empty());
}

#[tokio::test]
async fn a_group_is_renamed_for_all_its_skills_and_a_group_that_is_not_there_is_not_found() {
    let (service, _database) = service().await;
    service.place("skills/a.md", Some("Old")).await.unwrap();
    service.place("skills/b.md", Some("Old")).await.unwrap();
    let renamed = service.rename_group("Old", "New").await.unwrap();
    assert_eq!(groups(&renamed), [("New", "skills/a.md"), ("New", "skills/b.md")]);
    assert!(matches!(service.rename_group("Old", "Newer").await.unwrap_err(), RekallError::NotFound(_)));
    assert!(matches!(service.rename_group("New", "  ").await.unwrap_err(), RekallError::IllegalArgument(_)));
}
