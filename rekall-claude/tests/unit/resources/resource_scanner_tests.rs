use super::*;
use crate::resources::SkillSourceKind;

fn put(root: &Path, relative: &str, body: &str) {
    let target = root.join(relative);
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
    std::fs::write(target, body).unwrap();
}

fn names(summaries: &[ResourceSummary]) -> Vec<&str> {
    summaries.iter().map(|summary| summary.name.as_str()).collect()
}

fn catalog(root: &Path) -> ResourceCatalog {
    ResourceScanner::new(&ClaudeHome::new(root.to_path_buf())).catalog()
}

#[test]
fn a_folder_with_nothing_in_it_lists_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let found = catalog(dir.path());
    assert!(found.instructions.is_none() && found.skills.is_empty() && found.memory_groups.is_empty());
}

#[test]
fn a_folder_that_is_not_there_lists_nothing() {
    let dir = tempfile::tempdir().unwrap();
    assert!(catalog(&dir.path().join("missing")).skills.is_empty());
}

#[test]
fn skills_are_flat_files_and_folders_with_a_skill_file_and_buckets_are_searched() {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "skills/python-standards.md", "text");
    put(dir.path(), "skills/docx/SKILL.md", "text");
    put(dir.path(), "skills/docx/LICENSE.txt", "text");
    put(dir.path(), "skills/synced/bucket/pdf/SKILL.md", "text");
    put(dir.path(), "skills/synced/bucket/manifest.json", "{}");
    put(dir.path(), "skills/synced/notes.md", "ignored: not a skill beside a bucket");

    let found = catalog(dir.path());

    assert_eq!(names(&found.skills), ["docx", "pdf", "python-standards"]);
    assert!(!found.skills.iter().find(|skill| skill.name == "pdf").unwrap().editable);
}

#[test]
fn commands_and_agents_nest_by_folder() {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "commands/rk.md", "text");
    put(dir.path(), "commands/team/review.md", "text");
    put(dir.path(), "agents/planner.md", "text");
    put(dir.path(), "commands/.hidden.md", "text");

    let found = catalog(dir.path());

    assert_eq!(names(&found.commands), ["rk", "team:review"]);
    assert_eq!(names(&found.agents), ["planner"]);
}

#[test]
fn the_global_instructions_are_listed_when_present() {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "CLAUDE.md", "rules");
    assert_eq!(catalog(dir.path()).instructions.unwrap().bytes, 5);
}

#[test]
fn memories_are_grouped_per_project_with_the_index_apart() {
    let dir = tempfile::tempdir().unwrap();
    let work = dir.path().join("work-vega");
    std::fs::create_dir_all(&work).unwrap();
    let folder = crate::transcript::folder_name(&work);
    put(dir.path(), &format!("projects/{folder}/s1.jsonl"), &format!("{{\"cwd\":{:?}}}", work.to_string_lossy()));
    put(dir.path(), &format!("projects/{folder}/memory/MEMORY.md"), "- [A](a.md)\n");
    put(dir.path(), &format!("projects/{folder}/memory/a.md"), "---\nname: Alpha\n---\n");
    put(dir.path(), "projects/-gone-away/memory/b.md", "text");
    put(dir.path(), "projects/-empty/memory/MEMORY.md", "- nothing\n");
    put(dir.path(), "memory/everywhere.md", "text");

    let groups = catalog(dir.path()).memory_groups;

    assert_eq!(groups.iter().map(|group| group.id.as_str()).collect::<Vec<_>>(), ["global", "-gone-away", folder.as_str()]);
    let known = &groups[2];
    assert_eq!(names(&known.memories), ["Alpha"]);
    assert!(known.index.is_some() && known.resolved && known.folder_exists);
    assert!(!known.global);
    let orphan = &groups[1];
    assert!(!orphan.resolved && !orphan.folder_exists);
    assert!(groups[0].global && groups[0].working_directory.is_none());
}

#[test]
fn every_skill_says_whether_it_is_yours_claude_ais_or_a_plugins() {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "skills/mine.md", "text");
    put(dir.path(), "skills/synced/bucket/pdf/SKILL.md", "text");
    put(dir.path(), "skills/synced/bucket/manifest.json", r#"{"skills":[{"name":"pdf","source":"anthropic"}]}"#);
    put(dir.path(), "plugins/cache/official/design/2.0/skills/palette/SKILL.md", "text");
    let record = format!(
        r#"{{"version":2,"plugins":{{"design@official":[{{"installPath":"{}","version":"2.0"}}]}}}}"#,
        dir.path().join("plugins/cache/official/design/2.0").display()
    );
    put(dir.path(), "plugins/installed_plugins.json", &record);

    let found = catalog(dir.path());

    let source = |name: &str| found.skills.iter().find(|skill| skill.name == name).unwrap().source.clone().unwrap();
    assert_eq!(source("mine").kind, SkillSourceKind::Own);
    let synced = source("pdf");
    assert_eq!((synced.kind, synced.name.as_str(), synced.detail.as_deref()), (SkillSourceKind::Synced, "claude.ai", Some("anthropic")));
    let plugin = source("palette");
    assert_eq!((plugin.kind, plugin.name.as_str(), plugin.detail.as_deref()), (SkillSourceKind::Plugin, "design", Some("official · v2.0")));
    assert!(found.skills.iter().filter(|skill| !skill.editable).count() == 2);
}
