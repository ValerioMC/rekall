use super::*;

fn page(id: &str, title: &str, labels: &[&str], markdown: &str) -> ConfluencePage {
    ConfluencePage { id: id.into(), title: title.into(), labels: labels.iter().map(|label| label.to_string()).collect(), markdown: markdown.into(), url: format!("https://wiki/{id}") }
}

fn space(description: Option<&str>, home: Option<&str>) -> ConfluenceSpace {
    ConfluenceSpace { key: "ENG".into(), name: "Engineering".into(), description: description.map(str::to_string), home_page_id: home.map(str::to_string) }
}

#[test]
fn the_home_page_is_the_blueprint_and_no_section() {
    let pages = [page("1", "Home", &[], "Welcome to the platform."), page("2", "Release process", &[], "Steps")];

    let content = project_content(&space(None, Some("1")), &pages);

    assert_eq!(content.blueprint.as_deref(), Some("Welcome to the platform.\n\nSource: [Home](https://wiki/1)"));
    assert_eq!(content.sections.iter().map(|section| section.title.as_str()).collect::<Vec<_>>(), ["Deployment"]);
}

#[test]
fn the_space_description_wins_and_the_home_page_first_paragraph_stands_in_for_it() {
    let pages = [page("1", "Home", &[], "# Home\n\nThe platform that ships orders.\n\nMore")];

    assert_eq!(project_content(&space(Some("How we build"), Some("1")), &pages).description.as_deref(), Some("How we build"));
    assert_eq!(project_content(&space(None, Some("1")), &pages).description.as_deref(), Some("The platform that ships orders."));
    assert_eq!(project_content(&space(None, None), &pages).description, None);
}

#[test]
fn a_label_places_a_page_before_its_title_does() {
    let pages = [page("2", "Payments deployment notes", &["adr"], "x")];

    let content = project_content(&space(None, None), &pages);

    assert_eq!(content.sections.iter().map(|section| section.title.as_str()).collect::<Vec<_>>(), ["Decisions"]);
}

#[test]
fn pages_of_one_concept_share_a_section_and_sections_follow_a_fixed_order() {
    let pages = [
        page("2", "How to deploy", &[], "a"),
        page("3", "System architecture", &[], "b"),
        page("4", "Kubernetes environments", &[], "c"),
        page("5", "Team lunch", &[], "d"),
    ];

    let content = project_content(&space(None, None), &pages);

    let summary: Vec<(&str, usize)> = content.sections.iter().map(|section| (section.title.as_str(), section.pages)).collect();
    assert_eq!(summary, [("Architecture", 1), ("Deployment", 2), ("General", 1)]);
}

#[test]
fn a_short_keyword_matches_a_whole_word_only() {
    let pages = [page("2", "Decision log", &[], "a"), page("3", "Special cases", &[], "b")];

    let titles: Vec<String> = project_content(&space(None, None), &pages).sections.into_iter().map(|section| section.title).collect();

    assert_eq!(titles, ["Decisions", "General"]);
}

#[test]
fn a_section_lists_each_page_under_its_title_with_its_headings_lowered() {
    let pages = [page("2", "Runbook", &[], "# Steps\n\n```\n# not a heading\n```")];

    let body = &project_content(&space(None, None), &pages).sections[0].body;

    assert_eq!(body, "## Runbook\n\n[Open in Confluence](https://wiki/2)\n\n## Steps\n\n```\n# not a heading\n```");
}

#[test]
fn a_section_over_the_column_is_cut_with_a_note() {
    let pages = [page("2", "Runbook", &[], &"word ".repeat(30_000))];

    let body = &project_content(&space(None, None), &pages).sections[0].body;

    assert_eq!(body.chars().count(), BODY_MAX_CHARACTERS);
    assert!(body.ends_with("open it in Confluence._"));
}
