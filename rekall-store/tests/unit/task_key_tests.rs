use super::*;

fn taken(keys: &[&str]) -> HashSet<String> {
    keys.iter().map(|key| key.to_string()).collect()
}

#[test]
fn a_typed_key_is_capitalised_and_kept_to_two_to_four_characters() {
    assert_eq!(TaskKey::normalize(" rk ").unwrap(), "RK");
    for refused in ["r", "rekal", "1rk", "rk2", "r-k", ""] {
        assert!(TaskKey::normalize(refused).is_err(), "{refused}");
    }
}

#[test]
fn a_key_is_suggested_from_the_initials_of_the_label_when_it_has_several_words() {
    assert_eq!(TaskKey::suggest("rekall-vrust", &taken(&[])), "RV");
}

#[test]
fn a_one_word_label_gives_its_first_letters() {
    assert_eq!(TaskKey::suggest("rekall", &taken(&[])), "RE");
    assert_eq!(TaskKey::suggest("rekall", &taken(&["RE"])), "REK");
    assert_eq!(TaskKey::suggest("rekall", &taken(&["RE", "REK"])), "REKA");
}

#[test]
fn a_key_that_is_taken_everywhere_gets_one_more_letter() {
    assert_eq!(TaskKey::suggest("rekall", &taken(&["RE", "REK", "REKA"])), "REKB");
}

#[test]
fn a_label_with_no_letters_still_gets_a_key() {
    assert_eq!(TaskKey::suggest("2024", &taken(&[])), "PR");
}

#[test]
fn a_code_is_the_key_a_dash_and_the_number() {
    assert_eq!(TaskKey::code("RK", 12), "RK-12");
}
