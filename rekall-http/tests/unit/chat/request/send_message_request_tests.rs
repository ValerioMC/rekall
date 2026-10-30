use super::*;

fn request(text: Option<&str>) -> SendMessageRequest {
    SendMessageRequest { text: text.map(str::to_string) }
}

#[test]
fn the_message_is_trimmed() {
    assert_eq!(request(Some("  go on \n")).trimmed_text().unwrap(), "go on");
}

#[test]
fn a_missing_or_blank_message_is_refused() {
    assert!(request(None).trimmed_text().is_err());
    assert!(request(Some("   \n")).trimmed_text().is_err());
}

#[test]
fn a_message_is_bounded() {
    let longest = "a".repeat(MESSAGE_MAX_CHARACTERS);

    assert!(request(Some(&longest)).trimmed_text().is_ok());
    assert!(request(Some(&format!("{longest}a"))).trimmed_text().is_err());
}

#[test]
fn the_blanks_around_a_message_do_not_count_against_the_bound() {
    let padded = format!("  {}  ", "a".repeat(MESSAGE_MAX_CHARACTERS));

    assert!(request(Some(&padded)).trimmed_text().is_ok());
}
