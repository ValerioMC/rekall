use rekall_kernel::Instant;

use super::*;

fn page(title: &str) -> project_section::Model {
    let now = Instant::now();
    project_section::Model {
        id: Id::random(),
        project_id: Id::random(),
        context_id: Id::random(),
        title: title.into(),
        body_markdown: String::new(),
        position: 0,
        created_at: now,
        updated_at: now,
    }
}

fn group(context: &str, pages: &[&str]) -> PageGroup {
    PageGroup { context: context.into(), pages: pages.iter().map(|title| title.to_string()).collect() }
}

#[test]
fn every_page_named_once_under_contexts_is_accepted_ignoring_case_and_spaces() {
    let sections = [page("Deployment"), page("Glossary")];

    let wanted = checked(&[group(" Operations ", &["deployment"]), group("Team", &[" Glossary "])], &sections).unwrap();

    assert_eq!(wanted.iter().map(|group| group.context.as_str()).collect::<Vec<_>>(), ["Operations", "Team"]);
    assert_eq!(wanted[0].page_ids, [sections[0].id]);
    assert_eq!(wanted[1].page_ids, [sections[1].id]);
}

#[test]
fn a_page_left_out_is_refused_with_its_title() {
    let sections = [page("Deployment"), page("Glossary")];

    let refused = checked(&[group("Operations", &["Deployment"])], &sections).unwrap_err();

    assert!(refused.message().contains("Glossary"), "{}", refused.message());
    assert!(refused.message().contains("Nothing was moved"), "{}", refused.message());
}

#[test]
fn a_page_named_twice_or_not_on_the_project_is_refused() {
    let sections = [page("Deployment")];

    let twice = checked(&[group("A", &["Deployment"]), group("B", &["deployment"])], &sections).unwrap_err();
    let unknown = checked(&[group("A", &["Deployment", "Ghost"])], &sections).unwrap_err();

    assert!(twice.message().contains("'Deployment' is named twice"), "{}", twice.message());
    assert!(unknown.message().contains("no page titled 'Ghost'"), "{}", unknown.message());
}

#[test]
fn no_context_named_twice_and_none_blank() {
    let sections = [page("Deployment")];

    assert!(checked(&[], &sections).is_err());
    assert!(checked(&[group("  ", &["Deployment"])], &sections).unwrap_err().message().contains("needs a title"));
    let twice = checked(&[group("Ops", &["Deployment"]), group("ops", &[])], &sections).unwrap_err();
    assert!(twice.message().contains("named twice"), "{}", twice.message());
}
