use pulldown_cmark::CodeBlockKind;

pub(super) fn is_mermaid(kind: &CodeBlockKind<'_>) -> bool {
    matches!(kind, CodeBlockKind::Fenced(info) if info.split_whitespace().next() == Some("mermaid"))
}
