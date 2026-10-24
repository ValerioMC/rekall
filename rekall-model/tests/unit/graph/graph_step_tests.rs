use super::*;

fn step(title: &str, body: Option<&str>) -> Model {
    Model {
        id: Id::random(),
        graph_id: Id::random(),
        position: 0,
        title: title.into(),
        body_markdown: body.map(str::to_string),
        connection_id: None,
        max_visits: None,
        created_at: Instant::now(),
    }
}

#[test]
fn a_titled_step_without_detail_is_valid() {
    assert!(step("Push the commit", None).validate().is_ok());
}

#[test]
fn a_blank_title_is_refused() {
    assert!(step("  ", None).validate().is_err());
}

#[test]
fn detail_past_a_task_steps_limit_is_refused_since_it_becomes_one() {
    let long = "x".repeat(MAX_CHARACTERS + 1);
    assert!(step("Deploy", Some(&long)).validate().is_err());
}
