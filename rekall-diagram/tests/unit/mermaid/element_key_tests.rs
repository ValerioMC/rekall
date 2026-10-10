use super::*;

#[test]
fn a_hash_and_a_number_is_a_message() {
    assert_eq!(ElementKey::parse("#3"), Ok(ElementKey::Message(3)));
}

#[test]
fn anything_else_is_an_element_id() {
    assert_eq!(ElementKey::parse("OrderService"), Ok(ElementKey::Element("OrderService")));
}

#[test]
fn a_message_number_is_positive_and_numeric() {
    assert!(ElementKey::parse("#0").is_err());
    assert!(ElementKey::parse("#three").is_err());
    assert!(ElementKey::parse("#").is_err());
}
