use super::*;

#[test]
fn round_trips_through_its_text() {
    let id = Id::random();
    let text = id.to_string();
    assert_eq!(text.len(), 36);
    assert_eq!(text, text.to_lowercase());
    assert_eq!(text.parse::<Id>().unwrap(), id);
    assert_eq!(serde_json::to_string(&id).unwrap(), format!("\"{text}\""));
}
