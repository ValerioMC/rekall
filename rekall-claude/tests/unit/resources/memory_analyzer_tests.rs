use super::*;
use crate::resources::{ClaudeHome, CoveringKind};

fn put(root: &std::path::Path, relative: &str, body: &str) {
    let target = root.join(relative);
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
    std::fs::write(target, body).unwrap();
}

fn analysis(root: &std::path::Path) -> MemoryAnalysis {
    let home = ClaudeHome::new(root.to_path_buf());
    let catalog = crate::resources::ResourceScanner::new(&home).catalog();
    MemoryAnalyzer::new(&home).analyze(&catalog)
}

const MAKEFILE_RULE: &str = "Makefile targets must be phony, documented with a help target, and grouped by lifecycle phase: build, test, deploy.";

fn memory(body: &str) -> String {
    format!("---\nname: n\ndescription: d\nmetadata:\n  type: feedback\n---\n\n{body}\n")
}

#[test]
fn a_rule_repeated_in_several_projects_is_worth_promoting_to_the_instructions() {
    let dir = tempfile::tempdir().unwrap();
    for project in ["-work-alpha", "-work-beta"] {
        put(dir.path(), &format!("projects/{project}/memory/makefile.md"), &memory(MAKEFILE_RULE));
    }
    put(dir.path(), "projects/-work-alpha/memory/other.md", &memory("Database migration naming convention uses numbered prefixes everywhere"));
    put(dir.path(), "projects/-work-beta/memory/other.md", &memory("Frontend components stay under three hundred lines before splitting"));

    let found = analysis(dir.path());

    assert_eq!(found.promotions.len(), 1);
    let candidate = &found.promotions[0];
    assert_eq!((candidate.memories.len(), candidate.group_count, candidate.target), (2, 2, PromotionTarget::Instructions));
    assert!(candidate.topic.contains(&"makefile".to_string()));
}

#[test]
fn a_memory_repeated_inside_one_project_is_not_promoted() {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "projects/-work-alpha/memory/one.md", &memory(MAKEFILE_RULE));
    put(dir.path(), "projects/-work-alpha/memory/two.md", &memory(MAKEFILE_RULE));
    assert!(analysis(dir.path()).promotions.is_empty());
}

#[test]
fn a_recurring_procedure_is_suggested_as_a_skill() {
    let dir = tempfile::tempdir().unwrap();
    let steps = "Release checklist for the service:\n1. bump the version in the manifest\n2. tag the commit and push the tag\n3. wait for the pipeline to publish artifacts";
    put(dir.path(), "projects/-work-alpha/memory/release.md", &memory(steps));
    put(dir.path(), "projects/-work-beta/memory/release.md", &memory(steps));
    assert_eq!(analysis(dir.path()).promotions[0].target, PromotionTarget::Skill);
}

#[test]
fn a_memory_the_global_instructions_already_state_is_obsolete() {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "CLAUDE.md", &format!("# Rules\n\n- {MAKEFILE_RULE}\n- Other unrelated guidance entirely.\n"));
    put(dir.path(), "projects/-work-alpha/memory/makefile.md", &memory(MAKEFILE_RULE));

    let found = analysis(dir.path());

    assert_eq!(found.obsolete.len(), 1);
    assert_eq!(found.obsolete[0].covered_by.kind, CoveringKind::GlobalInstructions);
    assert_eq!(found.obsolete[0].covered_by.resource_path.as_deref(), Some("CLAUDE.md"));
    assert!(found.obsolete[0].overlap_percent >= 50);
    assert!(found.promotions.is_empty());
}

#[test]
fn a_memory_that_only_shares_vocabulary_with_a_skill_is_kept() {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "skills/make/SKILL.md", &format!("---\nname: make\n---\n\n{MAKEFILE_RULE} Plus lots more about targets, phases and help output.\n"));
    put(dir.path(), "projects/-work-alpha/memory/m.md", &memory("Deploy phase targets help the pipeline output when phony build runs lifecycle tests grouped."));
    assert!(analysis(dir.path()).obsolete.is_empty());
}

#[test]
fn a_memory_with_too_few_words_is_not_compared() {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "CLAUDE.md", "Use tabs.");
    put(dir.path(), "projects/-work-alpha/memory/tabs.md", &memory("Use tabs."));
    let found = analysis(dir.path());
    assert_eq!((found.memories_read, found.obsolete.len()), (0, 0));
}
