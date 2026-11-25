use super::*;
use crate::markdown::InlineRun;

fn part(markdown: &str) -> ExportPart {
    ExportPart { heading: "Part".to_string(), markdown: markdown.to_string() }
}

fn image(source: &str) -> DiagramImage {
    DiagramImage { source: source.to_string(), png: vec![1, 2, 3] }
}

fn text_of(runs: &[InlineRun]) -> String {
    runs.iter().map(|run| run.text.as_str()).collect()
}

fn heading_levels(blocks: &[Block]) -> Vec<u8> {
    blocks.iter().filter_map(|block| if let Block::Heading { level, .. } = block { Some(*level) } else { None }).collect()
}

#[test]
fn the_part_heading_comes_first_at_level_one() {
    let blocks = MarkdownBlocks::read(&part("Hello"), &[]);

    assert_eq!(blocks[0], Block::Heading { level: 1, inlines: vec![InlineRun::plain("Part")] });
    assert_eq!(blocks[1], Block::Paragraph(vec![InlineRun::plain("Hello")]));
}

#[test]
fn headings_move_one_level_down_and_stop_at_four() {
    let blocks = MarkdownBlocks::read(&part("# One\n\n## Two\n\n### Three\n\n###### Six"), &[]);

    assert_eq!(heading_levels(&blocks), vec![1, 2, 3, 4, 4]);
}

#[test]
fn a_table_keeps_its_header_and_rows() {
    let blocks = MarkdownBlocks::read(&part("| Name | Kind |\n|---|---|\n| a | b |\n| c | d |"), &[]);

    let Block::Table { header, rows } = &blocks[1] else { panic!("expected a table, got {:?}", blocks[1]) };
    assert_eq!(header.iter().map(|cell| text_of(cell)).collect::<Vec<_>>(), vec!["Name", "Kind"]);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[1].iter().map(|cell| text_of(cell)).collect::<Vec<_>>(), vec!["c", "d"]);
}

#[test]
fn a_nested_list_holds_its_items_inside_the_item() {
    let blocks = MarkdownBlocks::read(&part("1. first\n   - inner a\n   - inner b\n2. second"), &[]);

    let Block::List { ordered, items } = &blocks[1] else { panic!("expected a list, got {:?}", blocks[1]) };
    assert!(*ordered);
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].blocks[0], Block::Paragraph(vec![InlineRun::plain("first")]));
    let Block::List { ordered: inner_ordered, items: inner } = &items[0].blocks[1] else { panic!("expected a nested list") };
    assert!(!*inner_ordered);
    assert_eq!(inner.len(), 2);
    assert_eq!(items[1].blocks[0], Block::Paragraph(vec![InlineRun::plain("second")]));
}

#[test]
fn a_task_list_item_knows_whether_it_is_checked() {
    let blocks = MarkdownBlocks::read(&part("- [x] done\n- [ ] open"), &[]);

    let Block::List { items, .. } = &blocks[1] else { panic!("expected a list") };
    assert_eq!(items.iter().map(|item| item.checked).collect::<Vec<_>>(), vec![Some(true), Some(false)]);
}

#[test]
fn a_mermaid_fence_with_an_image_becomes_a_diagram() {
    let diagrams = [image("graph TD\n  A --> B")];

    let blocks = MarkdownBlocks::read(&part("```mermaid\ngraph TD\n  A --> B\n\n```"), &diagrams);

    assert_eq!(blocks[1], Block::Diagram { png: vec![1, 2, 3] });
}

#[test]
fn a_mermaid_fence_without_an_image_becomes_a_failed_diagram_and_never_its_source() {
    let blocks = MarkdownBlocks::read(&part("```mermaid\ngraph TD\n  A --> B\n```"), &[image("graph LR")]);

    assert_eq!(blocks[1], Block::DiagramFailed);
    assert_eq!(blocks.len(), 2);
}

#[test]
fn another_fence_stays_a_code_block_with_its_language() {
    let blocks = MarkdownBlocks::read(&part("```rust\nfn main() {}\n```"), &[]);

    assert_eq!(blocks[1], Block::CodeBlock { language: Some("rust".to_string()), text: "fn main() {}".to_string() });
}

#[test]
fn raw_html_is_dropped_and_its_text_kept() {
    let blocks = MarkdownBlocks::read(&part("<div>gone</div>\n\nHello <b>there</b>"), &[]);

    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[1], Block::Paragraph(vec![InlineRun::plain("Hello "), InlineRun::plain("there")]));
}

#[test]
fn an_image_is_written_as_its_alt_text() {
    let blocks = MarkdownBlocks::read(&part("![architecture sketch](docs/sketch.png)"), &[]);

    assert_eq!(blocks[1], Block::Paragraph(vec![InlineRun::plain("architecture sketch")]));
}

#[test]
fn inline_styles_and_links_are_carried_on_the_runs() {
    let blocks = MarkdownBlocks::read(&part("**bold** *italic* `code` [site](https://example.com)"), &[]);

    let Block::Paragraph(runs) = &blocks[1] else { panic!("expected a paragraph") };
    assert!(runs[0].bold && !runs[0].italic);
    assert!(runs[2].italic && !runs[2].bold);
    assert!(runs[4].code);
    assert_eq!(runs[6].link.as_deref(), Some("https://example.com"));
    assert_eq!(runs[6].text, "site");
}

#[test]
fn a_quote_holds_its_blocks() {
    let blocks = MarkdownBlocks::read(&part("> quoted\n\n---"), &[]);

    assert_eq!(blocks[1], Block::Quote(vec![Block::Paragraph(vec![InlineRun::plain("quoted")])]));
    assert_eq!(blocks[2], Block::Rule);
}

#[test]
fn mermaid_sources_lists_each_fence_trimmed_in_order() {
    let sources = MarkdownBlocks::mermaid_sources("```mermaid\n\ngraph A\n\n```\n\n```text\nnot this\n```\n\n- ```mermaid\n  graph B\n  ```");

    assert_eq!(sources, vec!["graph A".to_string(), "graph B".to_string()]);
}
