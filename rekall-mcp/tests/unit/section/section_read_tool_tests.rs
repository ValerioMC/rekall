use rekall_kernel::{Id, Instant};
use rekall_core::section::{ProjectSectionView, SectionsRead};

use super::{report, target_of};
use crate::tool::Anchor;

fn section(title: &str, body: &str) -> ProjectSectionView {
    let now = Instant::now();
    ProjectSectionView {
        id: Id::random(),
        project_id: Id::random(),
        title: title.into(),
        body_markdown: body.into(),
        position: 0,
        created_at: now,
        updated_at: now,
    }
}

fn read(sections: Vec<ProjectSectionView>, asked_for_one: bool) -> SectionsRead {
    SectionsRead { project_anchor: "project:vega".into(), sections, asked_for_one }
}

#[test]
fn without_a_title_the_sections_are_listed_with_their_size() {
    let listed = report(&read(vec![section("Deployment", "ship it"), section("Glossary", "")], false));

    assert!(listed.contains("2 sections"), "{listed}");
    assert!(listed.contains("- Deployment (7 characters)"), "{listed}");
    assert!(listed.contains("- Glossary (0 characters)"), "{listed}");
}

#[test]
fn a_project_without_sections_says_so() {
    assert_eq!(report(&read(vec![], false)), "`project:vega` has no sections.");
}

#[test]
fn a_section_asked_for_by_title_comes_back_whole() {
    let one = report(&read(vec![section("Deployment", "## Steps\nship it")], true));

    assert_eq!(one, "Section \"Deployment\" of `project:vega`:\n\n## Steps\nship it");
}

#[test]
fn a_project_or_a_task_names_where_to_read_and_nothing_else_does() {
    let project = target_of(&Anchor::parse_all(Some("project:vega")).unwrap()).unwrap();
    let task = target_of(&Anchor::parse_all(Some("project:vega task:report")).unwrap()).unwrap();

    assert_eq!((project.project_label.as_deref(), project.task_label), (Some("vega"), None));
    assert_eq!((task.project_label.as_deref(), task.task_label.as_deref()), (Some("vega"), Some("report")));
    assert!(target_of(&Anchor::parse_all(Some("company:acme")).unwrap()).is_err());
}
