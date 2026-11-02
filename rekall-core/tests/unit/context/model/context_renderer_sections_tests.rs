use super::*;

#[test]
fn a_project_without_sections_renders_no_block() {
    assert_eq!(ContextRenderer.render_sections(&[]), "");
}

#[test]
fn each_section_is_listed_by_title_with_how_to_read_it() {
    let rendered = ContextRenderer.render_sections(&["Contributing".to_string(), "Deployment".to_string()]);
    assert!(rendered.starts_with("\n<sections count=\"2\">\n"));
    assert!(rendered.contains("read that page with `rekall_read_sections` and follow it"));
    assert!(rendered.ends_with("- Contributing\n- Deployment\n</sections>\n"));
}
