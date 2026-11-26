use super::*;
use crate::document::ExportPicture;
use crate::markdown::{InlineRun, PictureAlignment, PictureBlock};

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
    let blocks = MarkdownBlocks::read(&part("Hello"), &[], &[]);

    assert_eq!(blocks[0], Block::Heading { level: 1, inlines: vec![InlineRun::plain("Part")] });
    assert_eq!(blocks[1], Block::Paragraph(vec![InlineRun::plain("Hello")]));
}

#[test]
fn headings_move_one_level_down_and_stop_at_four() {
    let blocks = MarkdownBlocks::read(&part("# One\n\n## Two\n\n### Three\n\n###### Six"), &[], &[]);

    assert_eq!(heading_levels(&blocks), vec![1, 2, 3, 4, 4]);
}

#[test]
fn a_table_keeps_its_header_and_rows() {
    let blocks = MarkdownBlocks::read(&part("| Name | Kind |\n|---|---|\n| a | b |\n| c | d |"), &[], &[]);

    let Block::Table { header, rows } = &blocks[1] else { panic!("expected a table, got {:?}", blocks[1]) };
    assert_eq!(header.iter().map(|cell| text_of(cell)).collect::<Vec<_>>(), vec!["Name", "Kind"]);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[1].iter().map(|cell| text_of(cell)).collect::<Vec<_>>(), vec!["c", "d"]);
}

#[test]
fn a_nested_list_holds_its_items_inside_the_item() {
    let blocks = MarkdownBlocks::read(&part("1. first\n   - inner a\n   - inner b\n2. second"), &[], &[]);

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
    let blocks = MarkdownBlocks::read(&part("- [x] done\n- [ ] open"), &[], &[]);

    let Block::List { items, .. } = &blocks[1] else { panic!("expected a list") };
    assert_eq!(items.iter().map(|item| item.checked).collect::<Vec<_>>(), vec![Some(true), Some(false)]);
}

#[test]
fn a_mermaid_fence_with_an_image_becomes_a_diagram() {
    let diagrams = [image("graph TD\n  A --> B")];

    let blocks = MarkdownBlocks::read(&part("```mermaid\ngraph TD\n  A --> B\n\n```"), &diagrams, &[]);

    assert_eq!(blocks[1], Block::Diagram { png: vec![1, 2, 3] });
}

#[test]
fn a_mermaid_fence_without_an_image_becomes_a_failed_diagram_and_never_its_source() {
    let blocks = MarkdownBlocks::read(&part("```mermaid\ngraph TD\n  A --> B\n```"), &[image("graph LR")], &[]);

    assert_eq!(blocks[1], Block::DiagramFailed);
    assert_eq!(blocks.len(), 2);
}

#[test]
fn another_fence_stays_a_code_block_with_its_language() {
    let blocks = MarkdownBlocks::read(&part("```rust\nfn main() {}\n```"), &[], &[]);

    assert_eq!(blocks[1], Block::CodeBlock { language: Some("rust".to_string()), text: "fn main() {}".to_string() });
}

#[test]
fn raw_html_is_dropped_and_its_text_kept() {
    let blocks = MarkdownBlocks::read(&part("<div>gone</div>\n\nHello <b>there</b>"), &[], &[]);

    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[1], Block::Paragraph(vec![InlineRun::plain("Hello "), InlineRun::plain("there")]));
}

fn picture(address: &str) -> ExportPicture {
    ExportPicture { address: address.to_string(), png: vec![9], natural_width: 40 }
}

fn placed(alignment: PictureAlignment) -> Block {
    Block::Picture(PictureBlock { png: vec![9], natural_width: 40, alignment })
}

fn read_with_picture(markdown: &str) -> Vec<Block> {
    MarkdownBlocks::read(&part(markdown), &[], &[picture("/pictures/flow.png")])
}

#[test]
fn a_picture_is_aligned_by_the_ending_of_its_address_and_stands_left_without_one() {
    for (ending, alignment) in [
        ("", PictureAlignment::Left),
        ("#left", PictureAlignment::Left),
        ("#center", PictureAlignment::Center),
        ("#right", PictureAlignment::Right),
        ("#other", PictureAlignment::Left),
    ] {
        let blocks = read_with_picture(&format!("![Flow](/pictures/flow.png{ending})"));

        assert_eq!(blocks[1..], [placed(alignment)], "ending '{ending}'");
    }
}

#[test]
fn a_picture_inside_a_line_of_text_splits_the_paragraph() {
    let blocks = read_with_picture("Before ![Flow](/pictures/flow.png) after");

    assert_eq!(
        blocks[1..],
        [Block::Paragraph(vec![InlineRun::plain("Before ")]), placed(PictureAlignment::Left), Block::Paragraph(vec![InlineRun::plain(" after")])]
    );
}

#[test]
fn pictures_side_by_side_leave_no_empty_paragraph_between_them() {
    let blocks = read_with_picture("![a](/pictures/flow.png) ![b](/pictures/flow.png#right)");

    assert_eq!(blocks[1..], [placed(PictureAlignment::Left), placed(PictureAlignment::Right)]);
}

#[test]
fn a_picture_that_was_not_handed_in_is_missing_with_its_whole_description() {
    let blocks = read_with_picture("![the **login** `flow`](/pictures/gone.png#center)");

    assert_eq!(blocks[1..], [Block::PictureMissing { description: "the login flow".to_string() }]);
}

#[test]
fn a_web_picture_is_printed_as_its_description_and_its_linked_address_within_the_line() {
    let blocks = read_with_picture("See ![Logo](https://example.com/logo.png#right) here");

    let Block::Paragraph(runs) = &blocks[1] else { panic!("expected a paragraph, got {:?}", blocks[1]) };
    assert_eq!(text_of(runs), "See Logo https://example.com/logo.png here");
    assert_eq!(runs[2].link.as_deref(), Some("https://example.com/logo.png"));
    assert_eq!(runs[3].link, None);
    assert_eq!(blocks.len(), 2);
}

#[test]
fn a_picture_in_a_table_cell_or_a_heading_is_printed_as_its_description() {
    let blocks = read_with_picture("## The ![Flow](/pictures/flow.png)\n\n| Shot |\n|---|\n| ![Flow](/pictures/flow.png) |");

    let Block::Heading { inlines, .. } = &blocks[1] else { panic!("expected a heading, got {:?}", blocks[1]) };
    let Block::Table { rows, .. } = &blocks[2] else { panic!("expected a table, got {:?}", blocks[2]) };
    assert_eq!(text_of(inlines), "The Flow");
    assert_eq!(text_of(&rows[0][0]), "Flow");
    assert_eq!(blocks.len(), 3);
}

#[test]
fn a_list_item_holds_its_picture() {
    let blocks = read_with_picture("- ![Flow](/pictures/flow.png)");

    let Block::List { items, .. } = &blocks[1] else { panic!("expected a list, got {:?}", blocks[1]) };
    assert_eq!(items[0].blocks, [placed(PictureAlignment::Left)]);
}

#[test]
fn local_picture_addresses_lose_their_ending_and_skip_the_web() {
    let addresses = MarkdownBlocks::local_picture_addresses(
        "![a](/pictures/a.png#right) ![w](https://example.com/w.png)\n\n```\n![not](/pictures/code.png)\n```\n\n| b |\n|---|\n| ![b](b.png) |",
    );

    assert_eq!(addresses, ["/pictures/a.png".to_string(), "b.png".to_string()]);
}

#[test]
fn inline_styles_and_links_are_carried_on_the_runs() {
    let blocks = MarkdownBlocks::read(&part("**bold** *italic* `code` [site](https://example.com)"), &[], &[]);

    let Block::Paragraph(runs) = &blocks[1] else { panic!("expected a paragraph") };
    assert!(runs[0].bold && !runs[0].italic);
    assert!(runs[2].italic && !runs[2].bold);
    assert!(runs[4].code);
    assert_eq!(runs[6].link.as_deref(), Some("https://example.com"));
    assert_eq!(runs[6].text, "site");
}

#[test]
fn a_quote_holds_its_blocks() {
    let blocks = MarkdownBlocks::read(&part("> quoted\n\n---"), &[], &[]);

    assert_eq!(blocks[1], Block::Quote(vec![Block::Paragraph(vec![InlineRun::plain("quoted")])]));
    assert_eq!(blocks[2], Block::Rule);
}

#[test]
fn mermaid_sources_lists_each_fence_trimmed_in_order() {
    let sources = MarkdownBlocks::mermaid_sources("```mermaid\n\ngraph A\n\n```\n\n```text\nnot this\n```\n\n- ```mermaid\n  graph B\n  ```");

    assert_eq!(sources, vec!["graph A".to_string(), "graph B".to_string()]);
}
