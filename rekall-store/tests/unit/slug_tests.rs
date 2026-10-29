use super::*;

#[test]
fn what_a_person_types_becomes_a_term_an_anchor_can_carry() {
    for (raw, expected) in [
        ("vega", "vega"),
        ("Vega", "vega"),
        ("  Vega Platform  ", "vega-platform"),
        ("Report Builder", "report-builder"),
        ("report-builder", "report-builder"),
        ("release_1.2", "release_1.2"),
        ("a  //  b", "a-b"),
        ("--edge--", "edge"),
        // An accent is dropped, not transliterated: a label is an identifier.
        ("caffè", "caff"),
    ] {
        assert_eq!(Slug::of(Some(raw)).unwrap(), expected, "{raw}");
    }
}

#[test]
fn the_result_is_always_a_valid_label_whatever_went_in() {
    for raw in ["Vega", "a / b", "..hidden..", "x--y", "1"] {
        assert!(Slug::matches(&Slug::of(Some(raw)).unwrap()), "{raw}");
    }
}

#[test]
fn a_value_with_nothing_usable_in_it_is_refused_rather_than_silently_emptied() {
    for raw in ["", "   ", "///", "...", "---"] {
        assert!(Slug::of(Some(raw)).unwrap_err().is_illegal_argument(), "{raw}");
    }
}

#[test]
fn refuses_null() {
    assert!(Slug::of(None).unwrap_err().is_illegal_argument());
}
