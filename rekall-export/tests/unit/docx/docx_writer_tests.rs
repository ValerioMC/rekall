use std::io::{Cursor, Read};

use super::*;
use crate::document::{DiagramImage, ExportPart};

const DIAGRAM_PNG: &[u8] = include_bytes!("../../fixtures/diagram.png");

fn document(parts: &[(&str, &str)], diagrams: Vec<DiagramImage>) -> ExportDocument {
    ExportDocument {
        title: "Rekall blueprint".to_string(),
        parts: parts.iter().map(|(heading, markdown)| ExportPart { heading: heading.to_string(), markdown: markdown.to_string() }).collect(),
        diagrams,
    }
}

fn entry(bytes: &[u8], name: &str) -> String {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).expect("the output is a zip");
    let mut text = String::new();
    archive.by_name(name).unwrap_or_else(|_| panic!("{name} is in the file")).read_to_string(&mut text).expect("the entry is text");
    text
}

fn entry_names(bytes: &[u8]) -> Vec<String> {
    let archive = zip::ZipArchive::new(Cursor::new(bytes)).expect("the output is a zip");
    archive.file_names().map(str::to_string).collect()
}

#[test]
fn headings_use_the_built_in_heading_styles() {
    let bytes = DocxWriter::write(&document(&[("Blueprint", "# Goals\n\n## Detail")], vec![])).unwrap();

    let xml = entry(&bytes, "word/document.xml");

    assert!(xml.contains(r#"<w:pStyle w:val="Heading1""#));
    assert!(xml.contains(r#"<w:pStyle w:val="Heading2""#));
    assert!(xml.contains(r#"<w:pStyle w:val="Heading3""#));
    assert!(xml.contains("Blueprint") && xml.contains("Goals") && xml.contains("Detail"));
    assert!(entry(&bytes, "word/styles.xml").contains(r#"w:styleId="Heading4""#));
}

#[test]
fn a_table_is_written_with_its_header_and_rows() {
    let bytes = DocxWriter::write(&document(&[("Data", "| Name | Kind |\n|---|---|\n| alpha | beta |")], vec![])).unwrap();

    let xml = entry(&bytes, "word/document.xml");

    assert!(xml.contains("<w:tbl>"));
    assert_eq!(xml.matches("<w:tr>").count() + xml.matches("<w:tr ").count(), 2);
    assert!(xml.contains("alpha") && xml.contains("Kind"));
}

#[test]
fn list_items_are_numbered_through_the_numbering_definitions() {
    let bytes = DocxWriter::write(&document(&[("Lists", "- first item\n  1. nested step\n- second item")], vec![])).unwrap();

    let xml = entry(&bytes, "word/document.xml");

    assert!(xml.contains("first item") && xml.contains("nested step") && xml.contains("second item"));
    assert!(xml.contains("<w:numPr>"));
    let numbering = entry(&bytes, "word/numbering.xml");
    assert!(numbering.contains(r#"w:val="bullet""#) && numbering.contains(r#"w:val="decimal""#));
}

#[test]
fn code_is_written_in_the_code_font() {
    let bytes = DocxWriter::write(&document(&[("Code", "```rust\nlet a = 1;\n    let b = 2;\n```")], vec![])).unwrap();

    let xml = entry(&bytes, "word/document.xml");

    assert!(xml.contains("Consolas"));
    assert!(xml.contains("let a = 1;") && xml.contains("let b = 2;"));
    assert!(xml.contains(r#"xml:space="preserve""#));
}

#[test]
fn a_diagram_with_an_image_is_an_inline_drawing_and_a_media_file() {
    let markdown = "```mermaid\ngraph TD\n  A --> B\n```";
    let diagrams = vec![DiagramImage { source: "graph TD\n  A --> B".to_string(), png: DIAGRAM_PNG.to_vec() }];

    let bytes = DocxWriter::write(&document(&[("Flow", markdown)], diagrams)).unwrap();

    let xml = entry(&bytes, "word/document.xml");
    assert!(xml.contains("<w:drawing>"));
    assert!(!xml.contains(DocumentStyle::DIAGRAM_ERROR_TEXT));
    assert!(entry_names(&bytes).iter().any(|name| name.starts_with("word/media/") && name.ends_with(".png")));
}

#[test]
fn a_diagram_without_an_image_is_the_error_paragraph_and_never_its_source() {
    let bytes = DocxWriter::write(&document(&[("Flow", "```mermaid\ngraph TD\n  A --> B\n```")], vec![])).unwrap();

    let xml = entry(&bytes, "word/document.xml");

    assert!(xml.contains(DocumentStyle::DIAGRAM_ERROR_TEXT));
    assert!(!xml.contains("graph TD"));
    assert!(!xml.contains("<w:drawing>"));
    assert!(xml.contains("<w:pBdr>") && xml.contains("<w:shd "));
}

#[test]
fn only_a_part_after_the_first_starts_a_new_page() {
    let one = DocxWriter::write(&document(&[("First", "text")], vec![])).unwrap();
    let two = DocxWriter::write(&document(&[("First", "text"), ("Second", "text")], vec![])).unwrap();

    assert_eq!(entry(&one, "word/document.xml").matches("<w:pageBreakBefore").count(), 0);
    assert_eq!(entry(&two, "word/document.xml").matches("<w:pageBreakBefore").count(), 1);
}

#[test]
fn the_header_carries_the_title_and_the_footer_a_page_number_field() {
    let bytes = DocxWriter::write(&document(&[("Part", "text")], vec![])).unwrap();

    let names = entry_names(&bytes);
    let header = names.iter().find(|name| name.starts_with("word/header")).expect("a header part");
    let footer = names.iter().find(|name| name.starts_with("word/footer")).expect("a footer part");

    assert!(entry(&bytes, header).contains("Rekall blueprint"));
    assert!(entry(&bytes, footer).contains("PAGE"));
}

#[test]
fn the_body_is_calibri_and_nothing_is_embedded() {
    let bytes = DocxWriter::write(&document(&[("Part", "text")], vec![])).unwrap();

    assert!(entry(&bytes, "word/styles.xml").contains("Calibri"));
    assert!(!entry_names(&bytes).iter().any(|name| name.contains("fonts/")));
}

#[test]
fn a_diagram_image_that_is_not_a_png_is_refused() {
    let diagrams = vec![DiagramImage { source: "graph TD".to_string(), png: b"not a png".to_vec() }];

    let failure = DocxWriter::write(&document(&[("Flow", "```mermaid\ngraph TD\n```")], diagrams));

    assert!(matches!(failure, Err(ExportError::Image(_))));
}
