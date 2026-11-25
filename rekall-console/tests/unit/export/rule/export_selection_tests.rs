use super::*;
use rekall_kernel::{Id, Instant};

fn page(title: &str, body: &str) -> project_section::Model {
    let now = Instant::now();
    project_section::Model {
        id: Id::random(), project_id: Id::random(), context_id: Id::random(), title: title.into(), body_markdown: body.into(),
        position: 0, created_at: now, updated_at: now,
    }
}

fn component(name: &str, summary: &str, description: &str) -> project_component::Model {
    let now = Instant::now();
    project_component::Model {
        id: Id::random(), project_id: Id::random(), name: name.into(), folder: format!("/work/{name}"), summary: summary.into(),
        description: description.into(), position: 0, created_at: now, updated_at: now,
    }
}

fn selection(blueprint: Option<&str>, pages: Vec<project_section::Model>, components: Vec<project_component::Model>) -> ExportSelection {
    ExportSelection { blueprint_markdown: blueprint.map(str::to_string), pages, components }
}

#[test]
fn the_parts_are_the_blueprint_then_the_pages_then_the_components() {
    let parts = selection(Some("Intro"), vec![page("Guide", "Steps")], vec![component("api", "", "Serves")]).parts();

    assert_eq!(parts.iter().map(|part| part.heading.as_str()).collect::<Vec<_>>(), ["Blueprint", "Guide", "api"]);
    assert_eq!(parts[1].markdown, "Steps");
}

#[test]
fn a_component_opens_with_its_folder_and_summary_before_its_description() {
    let parts = selection(None, vec![], vec![component("api", "The HTTP layer", "## Routes")]).parts();

    assert_eq!(parts[0].markdown, "`/work/api` — The HTTP layer\n\n## Routes");
}

#[test]
fn a_component_with_no_summary_has_only_its_folder_line() {
    let parts = selection(None, vec![], vec![component("api", "  ", "Body")]).parts();

    assert_eq!(parts[0].markdown, "`/work/api`\n\nBody");
}

#[test]
fn one_item_is_summarised_by_its_title() {
    assert_eq!(selection(Some("x"), vec![], vec![]).summary(), "Blueprint");
    assert_eq!(selection(None, vec![page("Guide", "")], vec![]).summary(), "Guide");
    assert_eq!(selection(None, vec![], vec![component("api", "", "")]).summary(), "api");
}

#[test]
fn several_items_are_summarised_by_their_counts() {
    let several = selection(Some("x"), vec![page("A", ""), page("B", ""), page("C", "")], vec![component("a", "", ""), component("b", "", "")]);

    assert_eq!(several.summary(), "Blueprint, 3 pages, 2 components");
    assert_eq!(selection(None, vec![page("A", "")], vec![component("a", "", "")]).summary(), "1 page, 1 component");
    assert_eq!(several.only_title(), None);
}
