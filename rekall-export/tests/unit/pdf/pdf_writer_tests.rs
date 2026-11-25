use std::collections::HashMap;

use lopdf::content::Content;
use lopdf::{Dictionary, Document, Object};

use super::*;
use crate::document::{DiagramImage, ExportPart};
use crate::style::DocumentStyle;

const DIAGRAM_PNG: &[u8] = include_bytes!("../../fixtures/diagram.png");

fn document(parts: &[(&str, &str)], diagrams: Vec<DiagramImage>) -> ExportDocument {
    ExportDocument {
        title: "Rekall blueprint".to_string(),
        parts: parts.iter().map(|(heading, markdown)| ExportPart { heading: heading.to_string(), markdown: markdown.to_string() }).collect(),
        diagrams,
    }
}

fn diagram(source: &str) -> DiagramImage {
    DiagramImage { source: source.to_string(), png: DIAGRAM_PNG.to_vec() }
}

fn unicode_map(pdf: &Document, font: &Dictionary) -> HashMap<u16, char> {
    let mut map = HashMap::new();
    let Ok(Object::Reference(id)) = font.get(b"ToUnicode") else { return map };
    let stream = pdf.get_object(*id).and_then(Object::as_stream).expect("the ToUnicode map is a stream");
    let text = String::from_utf8_lossy(&stream.decompressed_content().expect("the map inflates")).to_string();
    for line in text.lines().map(str::trim).filter(|line| line.starts_with('<') && line.matches('<').count() == 2) {
        let codes: Vec<&str> = line.split(['<', '>']).map(str::trim).filter(|part| !part.is_empty()).collect();
        if let [glyph, character] = codes[..] {
            if let (Ok(glyph), Ok(character)) = (u16::from_str_radix(glyph, 16), u32::from_str_radix(character, 16)) {
                map.extend(char::from_u32(character).map(|character| (glyph, character)));
            }
        }
    }
    map
}

fn decoded(bytes: &[u8], map: &HashMap<u16, char>) -> String {
    bytes.chunks(2).filter(|pair| pair.len() == 2).filter_map(|pair| map.get(&u16::from_be_bytes([pair[0], pair[1]]))).collect()
}

fn texts(bytes: &[u8]) -> Vec<String> {
    let pdf = Document::load_mem(bytes).expect("the output is a PDF");
    let mut texts = Vec::new();
    for page_id in pdf.get_pages().into_values() {
        let fonts = pdf.get_page_fonts(page_id).expect("the page lists its fonts");
        let maps: HashMap<Vec<u8>, HashMap<u16, char>> = fonts.iter().map(|(name, font)| (name.clone(), unicode_map(&pdf, font))).collect();
        let content = Content::decode(&pdf.get_page_content(page_id).expect("the page has content")).expect("the content decodes");
        let mut current: Option<&HashMap<u16, char>> = None;
        for operation in content.operations {
            match operation.operator.as_str() {
                "Tf" => current = operation.operands.first().and_then(|name| name.as_name().ok()).and_then(|name| maps.get(name)),
                "Tj" => {
                    if let (Some(map), Some(Ok(string))) = (current, operation.operands.first().map(Object::as_str)) {
                        texts.push(decoded(string, map));
                    }
                }
                _ => {}
            }
        }
    }
    texts
}

fn page_count(bytes: &[u8]) -> usize {
    Document::load_mem(bytes).expect("the output is a PDF").get_pages().len()
}

fn image_count(bytes: &[u8]) -> usize {
    let pdf = Document::load_mem(bytes).expect("the output is a PDF");
    pdf.objects.values().filter(|object| matches!(object, Object::Stream(stream) if stream.dict.get(b"Subtype").and_then(Object::as_name).ok() == Some(b"Image".as_slice()))).count()
}

#[test]
fn the_output_is_a_pdf() {
    let bytes = PdfWriter::write(&document(&[("Part", "text")], vec![])).unwrap();

    assert!(bytes.starts_with(b"%PDF-"));
    assert_eq!(page_count(&bytes), 1);
}

#[test]
fn the_heading_the_body_and_the_page_furniture_are_extractable() {
    let bytes = PdfWriter::write(&document(&[("Blueprint", "# Goals\n\nExport the pages as one document.")], vec![])).unwrap();

    let texts = texts(&bytes);

    assert!(texts.contains(&"Blueprint".to_string()));
    assert!(texts.contains(&"Goals".to_string()));
    assert!(texts.iter().any(|text| text == "Export the pages as one document."));
    assert!(texts.contains(&"Rekall blueprint".to_string()));
    assert!(texts.contains(&"1 / 1".to_string()));
}

#[test]
fn a_long_document_runs_onto_more_pages_and_numbers_them_out_of_the_total() {
    let markdown: String = (1..=120).map(|number| format!("Paragraph number {number} of the long part.\n\n")).collect();

    let bytes = PdfWriter::write(&document(&[("Long", &markdown)], vec![])).unwrap();

    let pages = page_count(&bytes);
    let texts = texts(&bytes);
    assert!(pages > 1);
    assert!(texts.contains(&format!("{pages} / {pages}")));
    assert!(texts.contains(&"Paragraph number 120 of the long part.".to_string()));
}

#[test]
fn only_a_part_after_the_first_starts_a_new_page() {
    let one = PdfWriter::write(&document(&[("First", "text")], vec![])).unwrap();
    let two = PdfWriter::write(&document(&[("First", "text"), ("Second", "text")], vec![])).unwrap();

    assert_eq!(page_count(&one), 1);
    assert_eq!(page_count(&two), 2);
}

#[test]
fn a_diagram_with_an_image_embeds_the_image_and_a_missing_one_prints_the_error_text() {
    let markdown = "```mermaid\ngraph TD\n  A --> B\n```\n\n```mermaid\ngraph LR\n  C --> D\n```";

    let bytes = PdfWriter::write(&document(&[("Flow", markdown)], vec![diagram("graph TD\n  A --> B")])).unwrap();

    let texts = texts(&bytes);
    assert_eq!(image_count(&bytes), 1);
    assert_eq!(texts.iter().filter(|text| text.as_str() == DocumentStyle::DIAGRAM_ERROR_TEXT).count(), 1);
    assert!(!texts.iter().any(|text| text.contains("graph")));
}

#[test]
fn a_diagram_image_that_is_not_a_png_is_refused() {
    let diagrams = vec![DiagramImage { source: "graph TD".to_string(), png: b"not a png".to_vec() }];

    let failure = PdfWriter::write(&document(&[("Flow", "```mermaid\ngraph TD\n```")], diagrams));

    assert!(matches!(failure, Err(ExportError::Image(_))));
}

#[test]
fn a_word_wider_than_the_page_is_wrapped_and_nothing_is_lost() {
    let long_word = "https://example.com/".to_string() + &"segment/".repeat(60) + "end";

    let bytes = PdfWriter::write(&document(&[("Links", &format!("See {long_word} for more."))], vec![])).unwrap();

    let texts = texts(&bytes);
    assert!(texts.iter().any(|text| text.starts_with("See https://")));
    assert!(texts.concat().contains(&format!("See {long_word} for more.")));
}

#[test]
fn tables_lists_and_code_are_written_with_their_text() {
    let markdown = "| Layer | Crate |\n|---|---|\n| store | rekall-store |\n\n- first item\n  1. nested step\n\n```rust\nfn main() {}\n```";

    let bytes = PdfWriter::write(&document(&[("Parts", markdown)], vec![])).unwrap();

    let texts = texts(&bytes);
    for expected in ["Layer", "Crate", "store", "rekall-store", "first item", "nested step", "fn main() {}", "1."] {
        assert!(texts.iter().any(|text| text == expected), "{expected} is in the file: {texts:?}");
    }
}

#[test]
fn a_link_is_an_annotation_on_the_page() {
    let bytes = PdfWriter::write(&document(&[("Links", "Read [the guide](https://example.com/guide) first.")], vec![])).unwrap();

    let pdf = Document::load_mem(&bytes).unwrap();
    let page_id = pdf.get_pages().into_values().next().unwrap();
    let page = pdf.get_dictionary(page_id).expect("the page is a dictionary");
    let annotations = pdf.dereference(page.get(b"Annots").expect("the page has annotations")).expect("the annotations resolve").1.as_array().expect("an array").clone();
    let targets: Vec<String> = annotations
        .iter()
        .filter_map(|annotation| pdf.dereference(annotation).ok()?.1.as_dict().ok())
        .filter_map(|annotation| pdf.dereference(annotation.get(b"A").ok()?).ok()?.1.as_dict().ok())
        .filter_map(|action| action.get(b"URI").ok()?.as_str().ok())
        .map(|uri| String::from_utf8_lossy(uri).to_string())
        .collect();
    assert_eq!(targets, vec!["https://example.com/guide".to_string()]);
}

#[test]
fn only_the_glyphs_used_are_embedded() {
    let bytes = PdfWriter::write(&document(&[("Small", "A short page of text with a **bold** word and `code`.")], vec![])).unwrap();

    assert!(bytes.len() < 150_000, "the file is {} bytes", bytes.len());
}
