use super::*;

#[test]
fn a_context_title_is_trimmed_and_must_say_something() {
    assert_eq!(title(Some("  Operations ")).unwrap(), "Operations");
    assert!(title(None).unwrap_err().message().contains("needs a title"));
    assert!(title(Some("   ")).unwrap_err().message().contains("needs a title"));
}

#[test]
fn a_context_title_past_the_column_is_refused_with_its_length() {
    let message = title(Some(&"t".repeat(201))).unwrap_err().message().to_string();
    assert!(message.contains("capped at 200 characters and this one is 201"), "{message}");
    assert!(title(Some(&"t".repeat(200))).is_ok());
}
