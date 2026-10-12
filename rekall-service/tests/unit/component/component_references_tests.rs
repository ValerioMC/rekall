use rekall_common::{Id, Instant};

use super::*;

fn component(name: &str, folder: &str, description: &str) -> ProjectComponentView {
    let now = Instant::now();
    ProjectComponentView {
        id: Id::random(),
        project_id: Id::random(),
        name: name.into(),
        folder: folder.into(),
        description: description.into(),
        position: 0,
        created_at: now,
        updated_at: now,
    }
}

fn expand(components: &[ProjectComponentView], text: &str) -> String {
    ComponentReferences::new(Some(components)).expand(text)
}

fn auth() -> Vec<ProjectComponentView> {
    vec![component("Auth Service", "services/auth", "Issues and checks tokens.")]
}

#[test]
fn a_reference_becomes_the_name_the_folder_and_the_description() {
    let out = expand(&auth(), "Use [[Auth Service]] here.");

    assert_eq!(out, "Use **Auth Service** (component, folder `services/auth`): Issues and checks tokens. here.");
}

#[test]
fn the_match_ignores_case_and_surrounding_spaces() {
    let out = expand(&auth(), "[[  auth service ]]");

    assert!(out.starts_with("**Auth Service** (component, folder `services/auth`)"), "{out}");
}

#[test]
fn a_repeated_reference_carries_only_name_and_folder() {
    let out = expand(&auth(), "[[Auth Service]] then [[Auth Service]]");

    assert_eq!(
        out,
        "**Auth Service** (component, folder `services/auth`): Issues and checks tokens. then \
         **Auth Service** (component, folder `services/auth`)"
    );
}

#[test]
fn the_description_is_given_once_across_the_texts_of_one_record() {
    let components = auth();
    let mut references = ComponentReferences::new(Some(&components));

    let first = references.expand("[[Auth Service]]");
    let second = references.expand("[[Auth Service]]");

    assert!(first.contains("Issues and checks tokens."));
    assert!(!second.contains("Issues and checks tokens."));
}

#[test]
fn a_component_without_a_description_or_folder_is_just_named() {
    let out = expand(&[component("Edge", "", "")], "[[Edge]]");

    assert_eq!(out, "**Edge** (component)");
}

#[test]
fn a_long_description_is_cut() {
    let long = "x".repeat(COMPONENT_REFERENCE_MAX + 50);

    let out = expand(&[component("Big", "big", &long)], "[[Big]]");

    assert_eq!(out, format!("**Big** (component, folder `big`): {}", "x".repeat(COMPONENT_REFERENCE_MAX)));
}

#[test]
fn an_unknown_name_is_flagged_not_dropped() {
    let out = expand(&auth(), "See [[Billing]].");

    assert_eq!(out, "See [[Billing]] (component not found in this project).");
}

#[test]
fn an_empty_project_flags_every_reference() {
    let out = expand(&[], "[[Auth Service]]");

    assert_eq!(out, "[[Auth Service]] (component not found in this project)");
}

#[test]
fn inline_code_is_left_untouched() {
    let text = "Write `[[Auth Service]]` and ``[[Auth Service]]`` literally.";

    assert_eq!(expand(&auth(), text), text);
}

#[test]
fn an_unclosed_backtick_does_not_hide_a_reference() {
    let out = expand(&auth(), "a ` b [[Auth Service]]");

    assert!(out.contains("**Auth Service**"), "{out}");
}

#[test]
fn a_fenced_block_is_left_untouched_and_the_text_after_it_is_expanded() {
    let text = "```\n[[Auth Service]]\n```\n[[Auth Service]]\n~~~md\n[[Auth Service]]\n~~~\n";

    let out = expand(&auth(), text);

    assert!(out.starts_with("```\n[[Auth Service]]\n```\n**Auth Service**"), "{out}");
    assert!(out.ends_with("~~~md\n[[Auth Service]]\n~~~\n"), "{out}");
}

#[test]
fn a_shorter_fence_does_not_close_a_longer_one() {
    let text = "````\n```\n[[Auth Service]]\n````\n";

    assert_eq!(expand(&auth(), text), text);
}

#[test]
fn text_with_no_reference_is_unchanged() {
    let text = "Plain text, a [link](x) and [single] brackets.\nSecond line";

    assert_eq!(expand(&auth(), text), text);
}

#[test]
fn an_opening_with_no_closing_is_left_as_written() {
    let text = "broken [[Auth Service and [[Auth\nService]] across lines";

    assert_eq!(expand(&auth(), text), text);
}

#[test]
fn a_blank_reference_is_left_as_written() {
    assert_eq!(expand(&auth(), "[[]] and [[   ]]"), "[[]] and [[   ]]");
}

#[test]
fn a_third_bracket_does_not_swallow_the_reference() {
    let out = expand(&auth(), "[[[Auth Service]]");

    assert!(out.starts_with("[**Auth Service**"), "{out}");
}

#[test]
fn a_record_with_no_project_passes_through() {
    let text = "[[Auth Service]]";

    assert_eq!(ComponentReferences::new(None).expand(text), text);
}

#[test]
fn accented_text_around_a_reference_survives() {
    let out = expand(&auth(), "è [[Auth Service]] à");

    assert!(out.starts_with("è **Auth Service**") && out.ends_with(" à"), "{out}");
}
