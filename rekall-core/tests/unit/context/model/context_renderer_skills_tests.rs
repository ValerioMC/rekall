use super::*;
use crate::context::{ContextSkillScope, ContextSkillView};

fn skill(name: &str, scope: ContextSkillScope) -> ContextSkillView {
    ContextSkillView { name: name.into(), arguments: None, scope }
}

fn skill_with(name: &str, arguments: &str, scope: ContextSkillScope) -> ContextSkillView {
    ContextSkillView { name: name.into(), arguments: Some(arguments.into()), scope }
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

#[test]
fn a_skill_with_arguments_prints_them_before_its_scope() {
    let rendered = ContextRenderer.render_skills(&[skill_with("impeccable", "redesign website ui", ContextSkillScope::Task)]);

    assert!(rendered.contains("- `impeccable` args: \"redesign website ui\" (task)\n"));
}

#[test]
fn a_skill_without_arguments_prints_no_args() {
    let rendered = ContextRenderer.render_skills(&[skill("impeccable", ContextSkillScope::Task)]);

    assert!(!rendered.contains("args:"));
}

#[test]
fn the_same_skill_on_task_and_step_with_different_arguments_is_two_invocations() {
    let skills = [
        skill_with("impeccable", "redesign", ContextSkillScope::Task),
        skill_with("impeccable", "polish", ContextSkillScope::Step("Final pass".into())),
    ];

    let rendered = ContextRenderer.render_skills(&skills);

    assert!(rendered.starts_with("\n<skills count=\"2\">\n"));
    assert!(rendered.contains("- `impeccable` args: \"redesign\" (task)\n"));
    assert!(rendered.contains("- `impeccable` args: \"polish\" (step: Final pass)\n"));
}

#[test]
fn quotes_and_line_breaks_in_arguments_are_escaped_so_the_text_survives() {
    let rendered = ContextRenderer.render_skills(&[skill_with("impeccable", "say \"hi\"\nthen stop", ContextSkillScope::Task)]);

    assert!(rendered.contains("args: \"say \\\"hi\\\"\\nthen stop\""));
}

#[test]
fn the_block_tells_the_session_to_pass_the_arguments_as_args() {
    let rendered = ContextRenderer.render_skills(&[skill_with("impeccable", "redesign", ContextSkillScope::Task)]);

    assert!(rendered.contains("pass that text"));
    assert!(rendered.contains("tool's args"));
}
