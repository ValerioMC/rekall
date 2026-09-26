use super::*;

fn parse(raw: &str) -> Vec<Anchor> {
    Anchor::parse_all(Some(raw)).unwrap()
}

fn failure(raw: &str) -> String {
    match Anchor::parse_all(Some(raw)).unwrap_err() {
        ToolError::Failure(message) => message,
        other => panic!("{other:?}"),
    }
}

#[test]
fn the_canonical_form_splits_into_entity_and_value() {
    assert_eq!(parse("project:vega task:report-builder"), [Anchor::new(Some("project"), "vega"), Anchor::new(Some("task"), "report-builder")]);
}

#[test]
fn a_term_without_a_qualifier_is_positional() {
    assert_eq!(parse("vega report-builder"), [Anchor::new(None, "vega"), Anchor::new(None, "report-builder")]);
}

#[test]
fn the_two_forms_mix_in_one_request() {
    assert_eq!(parse("vega task:report-builder"), [Anchor::new(None, "vega"), Anchor::new(Some("task"), "report-builder")]);
}

#[test]
fn a_quoted_value_keeps_its_spaces() {
    assert_eq!(parse("task:\"report builder\" project:vega"), [Anchor::new(Some("task"), "report builder"), Anchor::new(Some("project"), "vega")]);
}

#[test]
fn a_colon_inside_a_quoted_value_belongs_to_the_value() {
    assert_eq!(parse("\"ESA-4412: main workflow\""), [Anchor::new(None, "ESA-4412: main workflow")]);
}

#[test]
fn only_the_first_colon_qualifies_so_a_url_survives_as_a_value() {
    assert_eq!(parse("repo:https://gitlab.example/vega"), [Anchor::new(Some("repo"), "https://gitlab.example/vega")]);
}

#[test]
fn irregular_spacing_is_not_a_syntax_error() {
    assert_eq!(parse("  project:vega   task:report-builder \n"), [Anchor::new(Some("project"), "vega"), Anchor::new(Some("task"), "report-builder")]);
}

#[test]
fn an_empty_request_is_refused_rather_than_answered_with_everything() {
    assert!(failure("   ").contains("project:vega"));
}

#[test]
fn a_half_written_anchor_is_refused_rather_than_read_as_positional() {
    assert!(failure("project:").contains("not a valid anchor"));
    assert!(failure(":vega").contains("not a valid anchor"));
}
