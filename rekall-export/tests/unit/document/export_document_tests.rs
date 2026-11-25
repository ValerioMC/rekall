use super::*;

fn document(markdowns: &[&str], sources_with_image: &[&str]) -> ExportDocument {
    ExportDocument {
        title: "Export".to_string(),
        parts: markdowns.iter().map(|markdown| ExportPart { heading: "Part".to_string(), markdown: markdown.to_string() }).collect(),
        diagrams: sources_with_image.iter().map(|source| DiagramImage { source: source.to_string(), png: vec![0] }).collect(),
    }
}

#[test]
fn markdown_fences_without_an_image_are_counted_across_parts() {
    let document = document(&["```mermaid\ngraph A\n```\n\n```mermaid\ngraph B\n```", "```mermaid\ngraph C\n```"], &["graph B"]);

    assert_eq!(document.diagrams_missing(), 2);
}

#[test]
fn markdown_with_every_image_or_no_fence_counts_none() {
    let document = document(&["```mermaid\ngraph A\n```", "plain text"], &["graph A"]);

    assert_eq!(document.diagrams_missing(), 0);
}
