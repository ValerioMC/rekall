use super::*;

fn skill(name: &str, scope: ContextSkillScope) -> ContextSkillView {
    ContextSkillView { name: name.into(), scope }
}

#[test]
fn a_task_without_skills_renders_no_block() {
    assert_eq!(ContextRenderer.render_skills(&[]), "");
}

#[test]
fn each_skill_names_the_scope_that_assigned_it() {
    let skills = [
        skill("rust-standards", ContextSkillScope::Task),
        skill("code-review", ContextSkillScope::Tag("Backend".into())),
        skill("ui-ux-pro-max", ContextSkillScope::Step("Design the \"picker\"".into())),
    ];

    let rendered = ContextRenderer.render_skills(&skills);

    assert!(rendered.starts_with("\n<skills count=\"3\">\n"));
    assert!(rendered.contains("- `rust-standards` (task)\n"));
    assert!(rendered.contains("- `code-review` (tag: Backend)\n"));
    assert!(rendered.contains("- `ui-ux-pro-max` (step: Design the 'picker')\n"));
    assert!(rendered.ends_with("</skills>\n"));
}

#[test]
fn the_block_tells_the_session_to_invoke_the_skills() {
    let rendered = ContextRenderer.render_skills(&[skill("code-review", ContextSkillScope::Task)]);

    assert!(rendered.contains("Invoke each skill below with the Skill tool"));
}
