use rekall_store::document;

/// The step a note named `step-N-changes.md` refers to, read from the title alone.
pub fn step_number_of(note: &document::Model) -> Option<usize> {
    let title = note.title.trim().to_lowercase();
    title.strip_prefix("step-")?.strip_suffix("-changes.md")?.parse().ok()
}

pub fn deviation_steps(notes: &[document::Model]) -> Vec<usize> {
    let mut steps: Vec<usize> = notes.iter().filter_map(step_number_of).collect();
    steps.sort_unstable();
    steps
}

#[cfg(test)]
#[path = "../../../tests/unit/adherence/rule/deviation_note_rule_tests.rs"]
mod tests;
