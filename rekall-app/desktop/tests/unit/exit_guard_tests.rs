use super::*;

fn sessions(json: &str) -> Vec<SessionState> {
    serde_json::from_str(json).expect("a list of terminals")
}

#[test]
fn only_live_terminals_count_as_running_sessions() {
    let terminals = sessions(r#"[{"id":"a","live":true},{"id":"b","live":false},{"id":"c","live":true}]"#);
    assert_eq!(count_live(&terminals), 2);
}

#[test]
fn no_terminals_means_nothing_to_confirm() {
    assert_eq!(count_live(&sessions("[]")), 0);
}

#[test]
fn the_warning_names_one_session_in_the_singular() {
    let text = warning(1, Leaving::Quit);
    assert!(text.starts_with("1 Claude session is still running"));
    assert!(text.contains("so it will end"));
}

#[test]
fn the_warning_counts_several_sessions() {
    let text = warning(3, Leaving::Quit);
    assert!(text.starts_with("3 Claude sessions are still running"));
    assert!(text.contains("so they will end"));
}

#[test]
fn the_update_warning_says_the_restart_stops_the_server() {
    let text = warning(2, Leaving::Update);
    assert!(text.starts_with("2 Claude sessions are still running. Installing the update restarts Rekall"));
    assert!(text.contains("so they will end"));
}

#[tokio::test]
async fn the_consoles_answer_reaches_the_question_that_is_waiting() {
    let question = LeaveQuestion::default();
    let reply = question.open();
    assert!(question.answer(Some(true)));
    assert_eq!(reply.await.unwrap(), Some(true));
}

#[test]
fn an_answer_with_no_question_waiting_is_ignored() {
    assert!(!LeaveQuestion::default().answer(Some(true)));
}

#[tokio::test]
async fn a_new_question_abandons_the_one_still_waiting() {
    let question = LeaveQuestion::default();
    let abandoned = question.open();
    let _current = question.open();
    assert!(abandoned.await.is_err());
}

#[test]
fn the_console_is_told_what_to_say_and_a_deaf_console_defers_to_the_system() {
    let script = leave_script(Leaving::Quit, "2 sessions \"live\"");
    assert!(script.contains(r#""title":"Quit Rekall?""#));
    assert!(script.contains(r#""message":"2 sessions \"live\"""#));
    assert!(script.contains(r#""confirm":"Quit""#));
    assert!(script.contains("answerLeave(null)"));
}
