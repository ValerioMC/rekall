use super::*;

#[test]
fn installing_the_skill_packages_the_shared_workflow_without_claude_paths() {
    let home = tempfile::tempdir().unwrap();
    let skill = RekallSkill::under(home.path());
    assert!(!skill.installed());
    skill.install().unwrap();
    assert!(skill.installed());
    let body = fs::read_to_string(home.path().join("skills/rekall/SKILL.md")).unwrap();
    assert!(body.starts_with("---\nname: rekall\n"));
    assert!(body.contains("references/"));
    assert!(!body.contains(".claude/"));
    assert!(!body.contains("$ARGUMENTS"));
    assert!(!body.contains("Skill tool"));
    assert!(home.path().join("skills/rekall/references/replanning.md").is_file());
}
