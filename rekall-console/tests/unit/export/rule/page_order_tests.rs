use super::*;
use rekall_kernel::Instant;

fn context(title: &str, position: i32) -> section_context::Model {
    let now = Instant::now();
    section_context::Model { id: Id::random(), project_id: Id::random(), title: title.into(), position, created_at: now, updated_at: now }
}

fn page(title: &str, context: &section_context::Model, position: i32) -> project_section::Model {
    let now = Instant::now();
    project_section::Model {
        id: Id::random(), project_id: context.project_id, context_id: context.id, title: title.into(), body_markdown: String::new(),
        position, created_at: now, updated_at: now,
    }
}

#[test]
fn pages_follow_their_context_and_then_their_own_position() {
    let guides = context("Guides", 0);
    let decisions = context("Decisions", 1);
    let pages = vec![page("D2", &decisions, 1), page("G2", &guides, 1), page("D1", &decisions, 0), page("G1", &guides, 0)];

    let ordered = in_project_order(&[guides, decisions], pages);

    assert_eq!(ordered.iter().map(|page| page.title.as_str()).collect::<Vec<_>>(), ["G1", "G2", "D1", "D2"]);
}
