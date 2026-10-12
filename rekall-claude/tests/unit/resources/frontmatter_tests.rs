use super::*;

#[test]
fn name_description_and_a_top_level_type_are_read() {
    let front = Frontmatter::parse("---\nname: lean-visual\ndescription: few render rounds\ntype: feedback\n---\nBody");
    assert_eq!(front.name.as_deref(), Some("lean-visual"));
    assert_eq!(front.description.as_deref(), Some("few render rounds"));
    assert_eq!(front.kind.as_deref(), Some("feedback"));
}

#[test]
fn a_type_nested_under_metadata_is_found() {
    let front = Frontmatter::parse("---\nname: x\nmetadata:\n  node_type: memory\n  type: project\n---\n");
    assert_eq!(front.kind.as_deref(), Some("project"));
}

#[test]
fn quotes_around_a_value_are_dropped() {
    let front = Frontmatter::parse("---\ndescription: \"draw flows as mermaid\"\nname: 'quoted'\n---\n");
    assert_eq!(front.description.as_deref(), Some("draw flows as mermaid"));
    assert_eq!(front.name.as_deref(), Some("quoted"));
}

#[test]
fn a_folded_description_takes_its_indented_lines() {
    let front = Frontmatter::parse("---\nname: docs\ndescription: >\n  Use this when\n  editing docs.\nother: 1\n---\n");
    assert_eq!(front.description.as_deref(), Some("Use this when editing docs."));
}

#[test]
fn text_without_a_leading_fence_has_none() {
    assert_eq!(Frontmatter::parse("# Title\nname: not-front-matter\n"), Frontmatter::default());
    assert_eq!(Frontmatter::parse(""), Frontmatter::default());
}

#[test]
fn keys_after_the_closing_fence_are_body_text() {
    let front = Frontmatter::parse("---\nname: a\n---\nname: b\n");
    assert_eq!(front.name.as_deref(), Some("a"));
}

#[test]
fn a_byte_order_mark_does_not_hide_the_block() {
    assert_eq!(Frontmatter::parse("\u{feff}---\nname: a\n---\n").name.as_deref(), Some("a"));
}

#[test]
fn the_first_occurrence_of_a_key_wins() {
    assert_eq!(Frontmatter::parse("---\nname: first\nmetadata:\n  name: second\n---\n").name.as_deref(), Some("first"));
}
