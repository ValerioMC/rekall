use super::*;

#[test]
fn a_title_is_trimmed_and_must_say_something() {
    assert_eq!(title(Some("  Deployment ")).unwrap(), "Deployment");
    assert!(title(None).unwrap_err().message().contains("needs a title"));
    assert!(title(Some("   ")).unwrap_err().message().contains("needs a title"));
}

#[test]
fn a_title_past_the_column_is_refused_with_its_length() {
    let message = title(Some(&"t".repeat(201))).unwrap_err().message().to_string();
    assert!(message.contains("capped at 200 characters and this one is 201"), "{message}");
    assert!(title(Some(&"t".repeat(200))).is_ok());
}

#[test]
fn a_body_a_session_sends_is_stripped_and_cannot_be_empty() {
    assert_eq!(written_body(Some("\n# Notes\n\n")).unwrap(), "# Notes");
    assert!(written_body(Some("  \n")).unwrap_err().message().contains("needs a body"));
    assert!(written_body(None).unwrap_err().message().contains("needs a body"));
}

#[test]
fn a_body_past_the_column_is_told_to_split() {
    let message = written_body(Some(&"b".repeat(100_001))).unwrap_err().message().to_string();
    assert!(message.contains("capped at 100000 characters") && message.contains("Split it"), "{message}");
}
