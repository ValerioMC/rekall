use super::*;

fn home() -> (tempfile::TempDir, ClaudeHome) {
    let dir = tempfile::tempdir().unwrap();
    let home = ClaudeHome::new(dir.path().to_path_buf());
    (dir, home)
}

fn path(raw: &str) -> ResourcePath {
    ResourcePath::parse(raw).unwrap()
}

#[test]
fn writing_creates_the_folders_and_the_text_reads_back() {
    let (_dir, home) = home();
    home.write(&path("projects/-a/memory/rule.md"), "hello").unwrap();
    assert_eq!(home.read(&path("projects/-a/memory/rule.md")).unwrap(), "hello");
}

#[test]
fn writing_replaces_the_text_and_keeps_the_permissions() {
    let (dir, home) = home();
    let target = dir.path().join("CLAUDE.md");
    std::fs::write(&target, "old").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o644)).unwrap();
    }
    home.write(&path("CLAUDE.md"), "new").unwrap();
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "new");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(std::fs::metadata(&target).unwrap().permissions().mode() & 0o777, 0o644);
    }
}

#[cfg(unix)]
#[test]
fn a_symlinked_file_is_written_where_it_really_lives() {
    let (dir, home) = home();
    let elsewhere = tempfile::tempdir().unwrap();
    let real = elsewhere.path().join("dotfiles-claude.md");
    std::fs::write(&real, "old").unwrap();
    std::os::unix::fs::symlink(&real, dir.path().join("CLAUDE.md")).unwrap();

    home.write(&path("CLAUDE.md"), "new").unwrap();

    assert_eq!(std::fs::read_to_string(&real).unwrap(), "new");
    assert!(std::fs::symlink_metadata(dir.path().join("CLAUDE.md")).unwrap().file_type().is_symlink());
}

#[test]
fn a_file_that_is_not_there_is_not_found_and_has_no_version() {
    let (_dir, home) = home();
    assert!(matches!(home.read(&path("CLAUDE.md")), Err(RekallError::NotFound(_))));
    assert_eq!(home.modified_at(&path("CLAUDE.md")).unwrap(), None);
}

#[test]
fn a_file_that_is_not_text_is_refused() {
    let (dir, home) = home();
    std::fs::write(dir.path().join("CLAUDE.md"), [0xff, 0xfe, 0x00]).unwrap();
    assert!(home.read(&path("CLAUDE.md")).unwrap_err().is_illegal_argument());
}

#[test]
fn a_file_past_the_cap_is_refused() {
    let (dir, home) = home();
    std::fs::write(dir.path().join("CLAUDE.md"), vec![b'a'; MAX_BYTES as usize + 1]).unwrap();
    assert!(home.read(&path("CLAUDE.md")).unwrap_err().is_illegal_argument());
}

#[test]
fn the_head_stops_at_the_limit_without_splitting_a_character() {
    let (dir, home) = home();
    std::fs::write(dir.path().join("CLAUDE.md"), "aé").unwrap();
    assert_eq!(home.head(&path("CLAUDE.md"), 2).unwrap(), "a");
    assert_eq!(home.head(&path("CLAUDE.md"), 3).unwrap(), "aé");
}

#[test]
fn removing_the_last_file_of_a_skill_removes_its_folder() {
    let (dir, home) = home();
    home.write(&path("skills/docx/SKILL.md"), "x").unwrap();
    home.remove(&path("skills/docx/SKILL.md")).unwrap();
    assert!(!dir.path().join("skills/docx").exists());
    assert!(dir.path().join("skills").exists());
}

#[test]
fn a_skill_folder_with_other_files_stays() {
    let (dir, home) = home();
    home.write(&path("skills/docx/SKILL.md"), "x").unwrap();
    std::fs::write(dir.path().join("skills/docx/script.py"), "print()").unwrap();
    home.remove(&path("skills/docx/SKILL.md")).unwrap();
    assert!(dir.path().join("skills/docx/script.py").exists());
}

#[test]
fn removing_a_flat_skill_leaves_the_skills_folder() {
    let (dir, home) = home();
    home.write(&path("skills/only.md"), "x").unwrap();
    home.remove(&path("skills/only.md")).unwrap();
    assert!(dir.path().join("skills").is_dir());
}
