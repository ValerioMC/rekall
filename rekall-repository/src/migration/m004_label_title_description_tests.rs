use super::legacy_label;

#[test]
fn the_migrations_normalising_expression_turns_a_legacy_name_into_a_label() {
    for (legacy, expected) in [
        ("Vega", "vega"),
        ("Progetto Vega", "progetto-vega"),
        ("../../etc", "etc"),
        ("a/b/c", "a-b-c"),
    ] {
        assert_eq!(legacy_label(legacy), expected);
    }
}
