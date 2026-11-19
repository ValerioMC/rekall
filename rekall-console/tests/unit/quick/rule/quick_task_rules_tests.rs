use super::*;

#[test]
fn the_title_is_the_first_line_of_the_text() {
    assert_eq!(title("Fix the typo in README\nIt says recieve twice."), "Fix the typo in README");
}

#[test]
fn leading_blank_lines_do_not_make_an_empty_title() {
    assert_eq!(title("\n\n  Rename the flag  \nbecause it lies"), "Rename the flag");
}

#[test]
fn a_long_first_line_is_cut_at_eighty_characters() {
    let title = title(&"a".repeat(120));
    assert_eq!(title.chars().count(), 80);
}

#[test]
fn the_task_is_quick_and_its_description_is_the_whole_text() {
    let request = task_request(Id::random(), "  Fix it\nnow  ");
    assert!(request.quick);
    assert_eq!(request.title.as_deref(), Some("Fix it"));
    assert_eq!(request.description.as_deref(), Some("Fix it\nnow"));
}
