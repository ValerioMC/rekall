use super::*;

fn unit(project: Option<&str>, task: &str, step: Option<&str>) -> WorkUnit {
    WorkUnit::task(project, task).on_step(step)
}

#[test]
fn a_recorded_slash_command_names_its_task_and_step() {
    let text = "<command-message>rk</command-message>\n<command-name>/rk</command-name>\n<command-args>project:vega task:report step:3 start</command-args>";
    assert_eq!(unit_of_prompt(text), Some(unit(Some("vega"), "report", Some("3"))));
}

#[test]
fn a_typed_slash_line_counts_too() {
    assert_eq!(unit_of_prompt("/rk project:vega task:report wrapup"), Some(unit(Some("vega"), "report", None)));
}

#[test]
fn quoted_text_is_a_request_and_never_a_term() {
    let text = r#"/rk task:report note "the task:other and step:9 notes""#;
    assert_eq!(unit_of_prompt(text), Some(unit(None, "report", None)));
}

#[test]
fn a_prompt_that_is_not_an_rk_line_or_names_no_single_task_names_nothing() {
    assert_eq!(unit_of_prompt("please look at task:report"), None);
    assert_eq!(unit_of_prompt("/rk project:vega"), None);
    assert_eq!(unit_of_prompt("/rk task:one task:two"), None);
    assert_eq!(unit_of_prompt("<command-name>/other</command-name><command-args>task:report</command-args>"), None);
}

#[test]
fn anchors_name_a_task_and_never_a_step() {
    assert_eq!(unit_of_anchors("project:vega task:report"), Some(unit(Some("vega"), "report", None)));
    assert_eq!(unit_of_anchors("note:3f2a9c1e"), None);
}
