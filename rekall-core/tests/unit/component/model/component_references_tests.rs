use rekall_kernel::{Id, Instant};
use rekall_store::ConnectionKind;

use super::*;
use crate::task::TaskReferenceView;

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

fn connection(label: &str, title: &str) -> ConnectionView {
    ConnectionView {
        kind: ConnectionKind::Kubernetes,
        label: label.into(),
        title: title.into(),
        target: None,
        instance_url: None,
        credential_set: true,
    }
}

fn expand_with_connections(connections: &[ConnectionView], text: &str) -> String {
    ComponentReferences::new(Some(&[])).with_connections(connections).expand(text)
}

fn task(label: &str, title: &str) -> TaskReferenceView {
    TaskReferenceView { label: label.into(), title: title.into() }
}

fn expand_with_tasks(tasks: &[TaskReferenceView], text: &str) -> String {
    ComponentReferences::new(Some(&[])).with_tasks(tasks).expand(text)
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

#[test]
fn a_connection_reference_names_the_binding_and_points_at_its_line() {
    let out = expand_with_connections(&[connection("prod", "Production cluster")], "Deploy on [[connection:prod]].");

    assert_eq!(out, "Deploy on **prod** (connection \"Production cluster\", reached as its line in `<connections>` says).");
}

#[test]
fn a_connection_reference_ignores_case_and_surrounding_spaces() {
    let out = expand_with_connections(&[connection("prod", "Production cluster")], "[[ connection: PROD ]]");

    assert!(out.starts_with("**prod** (connection"), "{out}");
}

#[test]
fn an_unknown_connection_is_flagged_not_dropped() {
    let out = expand_with_connections(&[connection("prod", "Production cluster")], "[[connection:staging]]");

    assert_eq!(out, "[[connection:staging]] (connection not found in this project)");
}

#[test]
fn a_connection_reference_is_never_read_as_a_component() {
    let components = vec![component("connection:prod", "x", "")];

    let out = ComponentReferences::new(Some(&components)).expand("[[connection:prod]]");

    assert_eq!(out, "[[connection:prod]] (connection not found in this project)");
}

#[test]
fn a_task_reference_names_the_task_by_title_and_anchor() {
    let out = expand_with_tasks(&[task("export", "Export the report")], "Build on [[task:export]].");

    assert_eq!(out, "Build on **Export the report** (task `task:export`).");
}

#[test]
fn a_task_reference_ignores_case_and_surrounding_spaces() {
    let out = expand_with_tasks(&[task("export", "Export the report")], "[[ task: EXPORT ]]");

    assert_eq!(out, "**Export the report** (task `task:export`)");
}

#[test]
fn an_unknown_task_is_flagged_not_dropped() {
    let out = expand_with_tasks(&[task("export", "Export the report")], "[[task:import]]");

    assert_eq!(out, "[[task:import]] (task not found in this project)");
}

#[test]
fn the_task_labels_named_are_collected_lowercased_in_order() {
    let labels = referenced_task_labels("[[task:Second]] then [[Auth Service]] and [[ task:first ]]");

    assert_eq!(labels, vec!["second".to_string(), "first".to_string()]);
}

#[test]
fn a_task_named_inside_code_is_not_collected() {
    let labels = referenced_task_labels("`[[task:inline]]`\n```\n[[task:fenced]]\n```\n");

    assert!(labels.is_empty(), "{labels:?}");
}
