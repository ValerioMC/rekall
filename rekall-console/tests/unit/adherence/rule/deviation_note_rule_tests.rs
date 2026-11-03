use rekall_kernel::{Id, Instant};
use rekall_store::DocumentContextMode;

use super::*;

fn note(title: &str) -> document::Model {
    let now = Instant::now();
    document::Model {
        id: Id::random(),
        title: title.into(),
        kind: "notes".into(),
        body_markdown: String::new(),
        source_path: None,
        context_mode: DocumentContextMode::Full,
        scope_company_id: None,
        scope_project_id: None,
        created_at: now,
        updated_at: now,
    }
}

#[test]
fn a_note_titled_step_n_changes_names_step_n() {
    assert_eq!(step_number_of(&note("step-12-changes.md")), Some(12));
    assert_eq!(step_number_of(&note("Step-3-Changes.md")), Some(3));
}

#[test]
fn any_other_title_names_no_step() {
    for title in ["solution-overview.md", "step-changes.md", "step-x-changes.md", "step-3-changes", "my-step-3-changes.md"] {
        assert_eq!(step_number_of(&note(title)), None, "{title}");
    }
}

#[test]
fn the_steps_of_deviation_notes_come_sorted_and_skip_other_notes() {
    let notes = [note("step-9-changes.md"), note("piano.md"), note("step-2-changes.md")];

    assert_eq!(deviation_steps(&notes), vec![2, 9]);
}
