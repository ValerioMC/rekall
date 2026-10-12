use super::*;

fn summarize(raw: &str, body: &str) -> ResourceSummary {
    let dir = tempfile::tempdir().unwrap();
    let home = ClaudeHome::new(dir.path().to_path_buf());
    let path = ResourcePath::parse(raw).unwrap();
    home.write(&path, body).unwrap();
    ResourceSummary::read(&home, &path).unwrap()
}

#[test]
fn front_matter_names_and_describes_a_memory() {
    let summary = summarize("memory/a.md", "---\nname: lean\ndescription: few rounds\nmetadata:\n  type: feedback\n---\nBody");
    assert_eq!((summary.name.as_str(), summary.description.as_deref(), summary.memory_type.as_deref()), ("lean", Some("few rounds"), Some("feedback")));
    assert!(summary.editable);
}

#[test]
fn without_front_matter_the_name_comes_from_the_path() {
    assert_eq!(summarize("commands/rk.md", "text").name, "rk");
    assert_eq!(summarize("commands/team/review.md", "text").name, "team:review");
    assert_eq!(summarize("skills/python-standards.md", "text").name, "python-standards");
    assert_eq!(summarize("skills/docx/SKILL.md", "text").name, "docx");
    assert_eq!(summarize("CLAUDE.md", "text").name, "CLAUDE.md");
}

#[test]
fn without_a_description_the_first_prose_line_stands_in() {
    let summary = summarize("skills/x.md", "# Title\n\n```\ncode\n```\nThe first real sentence.\nSecond.");
    assert_eq!(summary.description.as_deref(), Some("The first real sentence."));
}

#[test]
fn the_stand_in_skips_front_matter_and_is_cut_short() {
    let long = "x".repeat(400);
    let summary = summarize("skills/y.md", &format!("---\nname: y\n---\n{long}"));
    assert_eq!(summary.description.unwrap().len(), 160);
}

#[test]
fn only_a_memory_carries_a_memory_type() {
    assert_eq!(summarize("commands/c.md", "---\ntype: x\n---\n").memory_type, None);
}

#[test]
fn the_size_and_a_version_are_reported() {
    let summary = summarize("CLAUDE.md", "12345");
    assert_eq!(summary.bytes, 5);
    assert_ne!(summary.modified_at, Instant::EPOCH);
}

#[test]
fn a_file_that_is_not_there_is_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let home = ClaudeHome::new(dir.path().to_path_buf());
    let error = ResourceSummary::read(&home, &ResourcePath::parse("CLAUDE.md").unwrap()).unwrap_err();
    assert!(matches!(error, rekall_common::RekallError::NotFound(_)));
}
