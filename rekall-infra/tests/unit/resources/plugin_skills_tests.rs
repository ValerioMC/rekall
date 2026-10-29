use super::*;
use crate::resources::SkillSourceKind;

fn put(root: &Path, relative: &str, body: &str) {
    let target = root.join(relative);
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
    std::fs::write(target, body).unwrap();
}

fn install(root: &Path, key: &str, install_path: &str, version: &str) {
    let record = format!(r#"{{"version":2,"plugins":{{"{key}":[{{"scope":"user","installPath":"{}","version":"{version}"}}]}}}}"#, root.join(install_path).display());
    put(root, "plugins/installed_plugins.json", &record);
}

#[test]
fn an_installed_plugins_skills_carry_the_plugin_and_its_marketplace() {
    let dir = tempfile::tempdir().unwrap();
    install(dir.path(), "design@official", "plugins/cache/official/design/2.0", "2.0");
    put(dir.path(), "plugins/cache/official/design/2.0/skills/palette/SKILL.md", "x");
    put(dir.path(), "plugins/cache/official/design/2.0/.claude/skills/fonts/SKILL.md", "x");
    put(dir.path(), "plugins/cache/official/design/1.0/skills/stale/SKILL.md", "an older version is not installed");

    let mut found = plugin_skill_files(dir.path());
    found.sort_by(|a, b| a.file.cmp(&b.file));

    assert_eq!(found.len(), 2);
    assert!(found[0].file.ends_with(".claude/skills/fonts/SKILL.md"));
    assert_eq!(found[0].source.kind, SkillSourceKind::Plugin);
    assert_eq!((found[0].source.name.as_str(), found[0].source.detail.as_deref()), ("design", Some("official · v2.0")));
}

#[test]
fn a_missing_or_unreadable_plugin_list_lists_nothing() {
    let dir = tempfile::tempdir().unwrap();
    assert!(plugin_skill_files(dir.path()).is_empty());
    put(dir.path(), "plugins/installed_plugins.json", "not json");
    assert!(plugin_skill_files(dir.path()).is_empty());
}

#[test]
fn an_install_outside_the_claude_folder_is_not_followed() {
    let dir = tempfile::tempdir().unwrap();
    let elsewhere = tempfile::tempdir().unwrap();
    put(elsewhere.path(), "skills/s/SKILL.md", "x");
    install(dir.path(), "p@m", &elsewhere.path().display().to_string(), "1");
    assert!(plugin_skill_files(dir.path()).is_empty());
}

#[test]
fn a_commit_hash_version_is_shortened_and_a_release_number_gets_a_v() {
    assert_eq!(readable_version("f359d43b4060"), "f359d43");
    assert_eq!(readable_version("2.0.1"), "v2.0.1");
    assert_eq!(detail_of("", Some("1.0.0")).as_deref(), Some("v1.0.0"));
    assert_eq!(detail_of("official", None).as_deref(), Some("official"));
}
