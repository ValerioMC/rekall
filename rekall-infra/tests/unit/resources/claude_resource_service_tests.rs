use super::*;

fn service() -> (tempfile::TempDir, ClaudeResourceService) {
    let dir = tempfile::tempdir().unwrap();
    let service = ClaudeResourceService::new(dir.path().to_path_buf());
    (dir, service)
}

fn create(path: &str, body: &str) -> ResourceSave {
    ResourceSave { path: Some(path.into()), body: Some(body.into()), expected_modified_at: None }
}

fn rewrite(opened: &ResourceFile, body: &str) -> ResourceSave {
    ResourceSave { path: Some(opened.summary.path.clone()), body: Some(body.into()), expected_modified_at: Some(opened.summary.modified_at) }
}

#[test]
fn a_new_file_is_created_and_comes_back_with_its_version() {
    let (_dir, service) = service();
    let saved = service.save(create("commands/hello.md", "---\ndescription: greets\n---\nHi")).unwrap();
    assert_eq!(saved.body, "---\ndescription: greets\n---\nHi");
    assert_eq!(saved.summary.description.as_deref(), Some("greets"));
}

#[test]
fn an_existing_file_is_rewritten_from_the_version_that_was_opened() {
    let (_dir, service) = service();
    let opened = service.save(create("CLAUDE.md", "one")).unwrap();
    let saved = service.save(rewrite(&opened, "two")).unwrap();
    assert_eq!(service.read("CLAUDE.md").unwrap().body, "two");
    assert_eq!(saved.body, "two");
}

#[test]
fn a_file_changed_behind_the_editor_is_a_conflict() {
    let (dir, service) = service();
    let opened = service.save(create("CLAUDE.md", "one")).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(20));
    std::fs::write(dir.path().join("CLAUDE.md"), "changed by a session").unwrap();

    let refused = service.save(rewrite(&opened, "mine")).unwrap_err();

    assert!(matches!(refused, RekallError::Conflict(_)));
    assert_eq!(service.read("CLAUDE.md").unwrap().body, "changed by a session");
}

#[test]
fn creating_over_a_file_that_exists_is_a_conflict() {
    let (_dir, service) = service();
    service.save(create("CLAUDE.md", "one")).unwrap();
    assert!(matches!(service.save(create("CLAUDE.md", "two")).unwrap_err(), RekallError::Conflict(_)));
}

#[test]
fn rewriting_a_file_that_was_removed_is_a_conflict() {
    let (dir, service) = service();
    let opened = service.save(create("CLAUDE.md", "one")).unwrap();
    std::fs::remove_file(dir.path().join("CLAUDE.md")).unwrap();
    assert!(matches!(service.save(rewrite(&opened, "two")).unwrap_err(), RekallError::Conflict(_)));
}

#[test]
fn a_body_past_the_cap_is_refused_before_anything_is_written() {
    let (dir, service) = service();
    let refused = service.save(create("CLAUDE.md", &"a".repeat(MAX_BODY_BYTES + 1))).unwrap_err();
    assert!(refused.is_illegal_argument());
    assert!(!dir.path().join("CLAUDE.md").exists());
}

#[test]
fn a_path_outside_the_known_places_is_refused() {
    let (dir, service) = service();
    assert!(service.save(create("../escape.md", "x")).unwrap_err().is_illegal_argument());
    assert!(service.read("settings.json").unwrap_err().is_illegal_argument());
    assert!(service.delete("../escape.md").unwrap_err().is_illegal_argument());
    assert!(!dir.path().parent().unwrap().join("escape.md").exists());
}

#[test]
fn a_skill_synced_from_claude_ai_cannot_be_written_or_deleted() {
    let (dir, service) = service();
    std::fs::create_dir_all(dir.path().join("skills/synced/b/pdf")).unwrap();
    std::fs::write(dir.path().join("skills/synced/b/pdf/SKILL.md"), "x").unwrap();
    assert!(matches!(service.save(create("skills/synced/b/pdf/SKILL.md", "y")).unwrap_err(), RekallError::Conflict(_)));
    assert!(matches!(service.delete("skills/synced/b/pdf/SKILL.md").unwrap_err(), RekallError::Conflict(_)));
    assert_eq!(service.read("skills/synced/b/pdf/SKILL.md").unwrap().body, "x");
}

#[test]
fn the_instructions_and_the_index_are_edited_but_never_deleted() {
    let (_dir, service) = service();
    service.save(create("CLAUDE.md", "x")).unwrap();
    service.save(create("memory/MEMORY.md", "x")).unwrap();
    assert!(matches!(service.delete("CLAUDE.md").unwrap_err(), RekallError::Conflict(_)));
    assert!(matches!(service.delete("memory/MEMORY.md").unwrap_err(), RekallError::Conflict(_)));
}

#[test]
fn a_new_memory_is_listed_in_the_index_beside_it() {
    let (_dir, service) = service();
    service.save(create("projects/-a/memory/MEMORY.md", "- [Old](old.md) — kept\n")).unwrap();

    service.save(create("projects/-a/memory/lean.md", "---\nname: Lean\ndescription: few rounds\n---\nBody")).unwrap();

    assert_eq!(service.read("projects/-a/memory/MEMORY.md").unwrap().body, "- [Old](old.md) — kept\n- [Lean](lean.md) — few rounds\n");
}

#[test]
fn the_first_memory_of_a_project_creates_the_index() {
    let (_dir, service) = service();
    service.save(create("projects/-a/memory/lean.md", "no front matter")).unwrap();
    assert_eq!(service.read("projects/-a/memory/MEMORY.md").unwrap().body, "- [lean](lean.md)\n");
}

#[test]
fn rewriting_a_memory_leaves_the_index_alone() {
    let (_dir, service) = service();
    let opened = service.save(create("memory/a.md", "---\nname: A\n---\n")).unwrap();
    let before = service.read("memory/MEMORY.md").unwrap().body;
    service.save(rewrite(&opened, "---\nname: A\ndescription: new\n---\n")).unwrap();
    assert_eq!(service.read("memory/MEMORY.md").unwrap().body, before);
}

#[test]
fn deleting_a_memory_removes_its_index_line() {
    let (_dir, service) = service();
    service.save(create("memory/a.md", "---\nname: A\n---\n")).unwrap();
    service.save(create("memory/b.md", "---\nname: B\n---\n")).unwrap();

    service.delete("memory/a.md").unwrap();

    assert!(matches!(service.read("memory/a.md").unwrap_err(), RekallError::NotFound(_)));
    assert_eq!(service.read("memory/MEMORY.md").unwrap().body, "- [B](b.md)\n");
}

#[test]
fn deleting_a_memory_with_no_index_succeeds() {
    let (dir, service) = service();
    std::fs::create_dir_all(dir.path().join("memory")).unwrap();
    std::fs::write(dir.path().join("memory/a.md"), "x").unwrap();
    service.delete("memory/a.md").unwrap();
    assert!(!dir.path().join("memory/MEMORY.md").exists());
}

#[test]
fn deleting_a_file_that_is_not_there_is_not_found() {
    let (_dir, service) = service();
    assert!(matches!(service.delete("commands/none.md").unwrap_err(), RekallError::NotFound(_)));
}

#[test]
fn the_catalog_reflects_the_disk() {
    let (_dir, service) = service();
    service.save(create("agents/planner.md", "x")).unwrap();
    assert_eq!(service.catalog().agents.len(), 1);
}
