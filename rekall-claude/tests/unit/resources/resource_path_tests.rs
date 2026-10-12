use super::*;

fn kind_of(raw: &str) -> ResourceKind {
    ResourcePath::parse(raw).unwrap().kind()
}

#[test]
fn each_place_claude_keeps_its_files_has_a_kind() {
    assert_eq!(kind_of("CLAUDE.md"), ResourceKind::Instructions);
    assert_eq!(kind_of("skills/python-standards.md"), ResourceKind::Skill);
    assert_eq!(kind_of("skills/docx/SKILL.md"), ResourceKind::Skill);
    assert_eq!(kind_of("skills/synced/bucket/pdf/SKILL.md"), ResourceKind::Skill);
    assert_eq!(kind_of("commands/rk.md"), ResourceKind::Command);
    assert_eq!(kind_of("commands/team/review.md"), ResourceKind::Command);
    assert_eq!(kind_of("agents/planner.md"), ResourceKind::Agent);
    assert_eq!(kind_of("memory/feedback.md"), ResourceKind::Memory);
    assert_eq!(kind_of("projects/-Users-me-vega/memory/rule.md"), ResourceKind::Memory);
    assert_eq!(kind_of("projects/-Users-me-vega/memory/MEMORY.md"), ResourceKind::MemoryIndex);
    assert_eq!(kind_of("memory/MEMORY.md"), ResourceKind::MemoryIndex);
}

#[test]
fn a_path_that_climbs_hides_or_leaves_the_known_places_is_refused() {
    for raw in [
        "",
        "../secrets.md",
        "skills/../../etc/passwd.md",
        "/etc/passwd.md",
        "skills//x.md",
        "skills/./x.md",
        "skills\\x.md",
        "skills/.hidden.md",
        "skills/x.txt",
        "skills/docx/notes.md",
        "settings.json",
        "history.jsonl",
        "projects/-Users-me/session.md",
        "projects/-Users-me/memory/nested/deep.md",
        "plugins/cache/x.md",
        "memory/bad\u{0}name.md",
        "memory/a:b.md",
        "CLAUDE.md/extra.md",
        "notes/CLAUDE.md",
    ] {
        assert!(ResourcePath::parse(raw).is_err(), "{raw:?} should be refused");
    }
}

#[test]
fn a_refusal_is_a_client_mistake() {
    assert!(ResourcePath::parse("../x.md").unwrap_err().is_illegal_argument());
}

#[test]
fn surrounding_spaces_are_ignored() {
    assert_eq!(ResourcePath::parse("  CLAUDE.md ").unwrap().as_str(), "CLAUDE.md");
}

#[test]
fn skills_synced_from_claude_ai_are_not_editable() {
    assert!(!ResourcePath::parse("skills/synced/bucket/pdf/SKILL.md").unwrap().is_editable());
    assert!(ResourcePath::parse("skills/pdf/SKILL.md").unwrap().is_editable());
}

#[test]
fn a_memory_knows_the_index_beside_it() {
    let memory = ResourcePath::parse("projects/-a-b/memory/rule.md").unwrap();
    assert_eq!(memory.memory_index().unwrap().as_str(), "projects/-a-b/memory/MEMORY.md");
    assert_eq!(ResourcePath::parse("memory/rule.md").unwrap().memory_index().unwrap().as_str(), "memory/MEMORY.md");
    assert!(ResourcePath::parse("commands/rk.md").unwrap().memory_index().is_none());
}

#[test]
fn the_file_name_is_the_last_segment() {
    assert_eq!(ResourcePath::parse("commands/team/review.md").unwrap().file_name(), "review.md");
}

#[test]
fn an_installed_plugins_skills_are_known_places_and_read_only() {
    for raw in ["plugins/cache/official/design/2.0/skills/palette/SKILL.md", "plugins/cache/official/design/2.0/.claude/skills/fonts/SKILL.md"] {
        let path = ResourcePath::parse(raw).unwrap();
        assert_eq!(path.kind(), ResourceKind::Skill);
        assert!(!path.is_editable(), "{raw}");
    }
}

#[test]
fn the_plugin_cache_opens_no_other_door() {
    for raw in [
        "plugins/installed_plugins.json",
        "plugins/cache/official/design/2.0/README.md",
        "plugins/cache/official/design/2.0/skills/palette/references/guide.md",
        "plugins/cache/official/design/2.0/.git/skills/x/SKILL.md",
        "skills/.claude/SKILL.md",
        "plugins/marketplaces/m/skills/x/SKILL.md",
    ] {
        assert!(ResourcePath::parse(raw).is_err(), "{raw}");
    }
}
