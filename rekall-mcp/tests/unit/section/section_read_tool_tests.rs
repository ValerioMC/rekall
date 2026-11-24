use rekall_kernel::{Id, Instant};
use rekall_core::section::{ProjectSectionView, SectionContextView, SectionsRead};
use rekall_core::ProjectRef;

use super::{report, target_of};
use crate::tool::Anchor;

fn context(title: &str) -> SectionContextView {
    let now = Instant::now();
    SectionContextView { id: Id::random(), project_id: Id::random(), title: title.into(), position: 0, created_at: now, updated_at: now }
}

fn section_in(context: &SectionContextView, title: &str, body: &str) -> ProjectSectionView {
    let now = Instant::now();
    ProjectSectionView {
        id: Id::random(),
        project_id: context.project_id,
        context_id: context.id,
        title: title.into(),
        body_markdown: body.into(),
        position: 0,
        created_at: now,
        updated_at: now,
    }
}

fn read(contexts: Vec<SectionContextView>, sections: Vec<ProjectSectionView>, asked_for_one: bool) -> SectionsRead {
    SectionsRead { project_anchor: "project:vega".into(), contexts, sections, asked_for_one }
}

#[test]
fn without_a_title_the_sections_are_listed_by_context_with_their_size() {
    let operations = context("Operations");
    let team = context("Team");
    let sections = vec![section_in(&operations, "Deployment", "ship it"), section_in(&team, "Glossary", "")];

    let listed = report(&read(vec![operations, team], sections, false));

    assert!(listed.contains("2 sections"), "{listed}");
    assert!(listed.contains("Operations:\n- Deployment (7 characters)"), "{listed}");
    assert!(listed.contains("Team:\n- Glossary (0 characters)"), "{listed}");
}

#[test]
fn a_context_with_no_section_is_left_out_of_the_list() {
    let operations = context("Operations");
    let empty = context("Empty");
    let sections = vec![section_in(&operations, "Deployment", "ship it")];

    let listed = report(&read(vec![operations, empty], sections, false));

    assert!(!listed.contains("Empty"), "{listed}");
}

#[test]
fn a_project_without_sections_says_so() {
    assert_eq!(report(&read(vec![], vec![], false)), "`project:vega` has no sections.");
}

#[test]
fn a_section_asked_for_by_title_comes_back_whole_with_its_context() {
    let operations = context("Operations");
    let sections = vec![section_in(&operations, "Deployment", "## Steps\nship it")];

    let one = report(&read(vec![operations], sections, true));

    assert_eq!(one, "Section \"Deployment\" of `project:vega`, in the context \"Operations\":\n\n## Steps\nship it");
}

#[test]
fn a_project_or_a_task_names_where_to_read_and_nothing_else_does() {
    let project = target_of(&Anchor::parse_all(Some("project:vega")).unwrap()).unwrap();
    let task = target_of(&Anchor::parse_all(Some("project:vega task:report")).unwrap()).unwrap();

    assert_eq!((project.project, project.task_code), (Some(ProjectRef::bare("vega")), None));
    assert_eq!((task.project, task.task_code.as_deref()), (Some(ProjectRef::bare("vega")), Some("report")));
    assert!(target_of(&Anchor::parse_all(Some("company:acme")).unwrap()).is_err());
}
