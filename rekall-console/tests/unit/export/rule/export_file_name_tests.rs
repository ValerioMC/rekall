use super::*;

#[test]
fn the_name_is_the_project_label_then_what_was_exported_in_lower_case() {
    assert_eq!(of_export("vega", Some("Release Plan: Q4!"), ExportFormat::Pdf), "vega-release-plan-q4.pdf");
    assert_eq!(of_export("vega", Some("Blueprint"), ExportFormat::Docx), "vega-blueprint.docx");
}

#[test]
fn several_items_or_a_title_with_no_safe_character_are_called_export() {
    assert_eq!(of_export("vega", None, ExportFormat::Pdf), "vega-export.pdf");
    assert_eq!(of_export("vega", Some("???"), ExportFormat::Pdf), "vega-export.pdf");
}

#[test]
fn a_long_title_is_cut_so_the_name_stays_within_its_column() {
    let name = of_export(&"p".repeat(120), Some(&"word ".repeat(100)), ExportFormat::Docx);

    assert!(name.chars().count() <= 255);
    assert!(!name.contains("-."));
}
