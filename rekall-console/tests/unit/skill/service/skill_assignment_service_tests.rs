use super::*;
use rekall_core::EventBus;
use rekall_store::enums::ProjectStatus;
use rekall_store::{company, project, tag, task};
use sea_orm::{ActiveModelTrait, IntoActiveModel};

fn skill(name: &str, arguments: Option<&str>) -> SkillCommand {
    SkillCommand { name: name.to_string(), arguments: arguments.map(str::to_string) }
}

fn names(values: &[&str]) -> Vec<SkillCommand> {
    values.iter().map(|value| skill(value, None)).collect()
}

#[test]
fn names_keep_their_order_and_lose_repeats() {
    let distinct = distinct_skills(names(&["code-review", " ui-ux-pro-max ", "code-review"])).unwrap();

    assert_eq!(distinct, names(&["code-review", "ui-ux-pro-max"]));
}

#[test]
fn a_name_given_twice_keeps_its_first_arguments() {
    let distinct = distinct_skills(vec![skill("impeccable", Some("redesign")), skill("impeccable", Some("polish"))]).unwrap();

    assert_eq!(distinct, [skill("impeccable", Some("redesign"))]);
}

#[test]
fn arguments_are_trimmed_and_blank_ones_dropped() {
    let distinct = distinct_skills(vec![skill("impeccable", Some("  redesign website ui ")), skill("rk", Some("   "))]).unwrap();

    assert_eq!(distinct, [skill("impeccable", Some("redesign website ui")), skill("rk", None)]);
}

#[test]
fn arguments_over_the_cap_are_refused() {
    let too_long = "a".repeat(skill_arguments::SKILL_ARGUMENTS_MAX + 1);

    assert!(distinct_skills(vec![skill("impeccable", Some(&too_long))]).unwrap_err().is_illegal_argument());
}

#[test]
fn a_plugin_qualified_name_is_accepted() {
    assert_eq!(distinct_skills(names(&["code-review:code-review"])).unwrap(), names(&["code-review:code-review"]));
}

#[test]
fn a_name_with_a_space_or_a_slash_is_refused() {
    assert!(distinct_skills(names(&["two words"])).unwrap_err().is_illegal_argument());
    assert!(distinct_skills(names(&["../escape"])).unwrap_err().is_illegal_argument());
    assert!(distinct_skills(names(&[""])).unwrap_err().is_illegal_argument());
}

#[test]
fn more_skills_than_the_cap_are_refused() {
    let many: Vec<SkillCommand> = (0..=SKILLS_PER_OWNER_MAX).map(|index| skill(&format!("skill-{index}"), None)).collect();

    assert!(distinct_skills(many).unwrap_err().is_illegal_argument());
}

#[test]
fn an_empty_list_clears_the_owner() {
    assert!(distinct_skills(Vec::new()).unwrap().is_empty());
}

mod stored {
    use super::*;

    async fn service() -> (SkillAssignmentService, Ctx) {
        let database = rekall_store::open_in_memory().await.unwrap();
        let ctx = Ctx::new(database.conn.clone(), EventBus::new());
        (SkillAssignmentService::new(ctx.clone()), ctx)
    }

    async fn a_tag(ctx: &Ctx) -> tag::Model {
        let now = ctx.now();
        let tag = tag::Model { id: Id::random(), name: "frontend".into(), icon: "tag".into(), color: "blue".into(), created_at: now, updated_at: now };
        tag.clone().into_active_model().insert(&ctx.db).await.unwrap();
        tag
    }

    async fn a_task(ctx: &Ctx) -> task::Model {
        let now = ctx.now();
        let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
        company.clone().into_active_model().insert(&ctx.db).await.unwrap();
        let project = project::Model {
            id: Id::random(), label: "vega".into(), title: "Vega".into(), status: ProjectStatus::Active, icon: "folder".into(),
            description: None, blueprint_markdown: None, repo_folder: None, auto_commit: false, main_branch: None, pull_request_flow: false, task_key: String::new(), company_id: company.id,
            created_at: now, updated_at: now,
        };
        project.clone().into_active_model().insert(&ctx.db).await.unwrap();
        let task = task::Model::new("report".into(), "Report".into(), project.id, now);
        task.clone().into_active_model().insert(&ctx.db).await.unwrap();
        task
    }

    #[tokio::test]
    async fn the_arguments_of_a_tag_skill_are_stored_and_returned_by_the_listing() {
        let (service, ctx) = service().await;
        let tag = a_tag(&ctx).await;

        let written = service.set_for_tag(tag.id, vec![skill("impeccable", Some(" redesign website ui ")), skill("rk", None)]).await.unwrap();

        let listed = service.list().await.unwrap().tag_skills;
        assert_eq!(written, listed);
        let pairs: Vec<(&str, Option<&str>)> = listed.iter().map(|view| (view.skill_name.as_str(), view.arguments.as_deref())).collect();
        assert_eq!(pairs, [("impeccable", Some("redesign website ui")), ("rk", None)]);
    }

    #[tokio::test]
    async fn a_write_replaces_the_arguments_with_the_list() {
        let (service, ctx) = service().await;
        let task = a_task(&ctx).await;
        service.set_for_task(task.id, vec![skill("impeccable", Some("redesign"))]).await.unwrap();

        service.set_for_task(task.id, vec![skill("impeccable", Some("polish"))]).await.unwrap();

        let listed = service.list().await.unwrap().task_skills;
        assert_eq!(listed.iter().map(|view| view.arguments.as_deref()).collect::<Vec<_>>(), [Some("polish")]);
    }

    #[tokio::test]
    async fn arguments_over_the_cap_write_nothing() {
        let (service, ctx) = service().await;
        let task = a_task(&ctx).await;
        service.set_for_task(task.id, vec![skill("impeccable", Some("redesign"))]).await.unwrap();

        let refused = service.set_for_task(task.id, vec![skill("impeccable", Some(&"a".repeat(301)))]).await.unwrap_err();

        assert!(refused.is_illegal_argument());
        let listed = service.list().await.unwrap().task_skills;
        assert_eq!(listed.iter().map(|view| view.arguments.as_deref()).collect::<Vec<_>>(), [Some("redesign")]);
    }
}
