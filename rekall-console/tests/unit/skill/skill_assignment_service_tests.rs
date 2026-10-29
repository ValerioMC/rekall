use super::*;

fn names(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| value.to_string()).collect()
}

#[test]
fn names_keep_their_order_and_lose_repeats() {
    let distinct = distinct_names(names(&["code-review", " ui-ux-pro-max ", "code-review"])).unwrap();

    assert_eq!(distinct, ["code-review", "ui-ux-pro-max"]);
}

#[test]
fn a_plugin_qualified_name_is_accepted() {
    assert_eq!(distinct_names(names(&["code-review:code-review"])).unwrap(), ["code-review:code-review"]);
}

#[test]
fn a_name_with_a_space_or_a_slash_is_refused() {
    assert!(distinct_names(names(&["two words"])).unwrap_err().is_illegal_argument());
    assert!(distinct_names(names(&["../escape"])).unwrap_err().is_illegal_argument());
    assert!(distinct_names(names(&[""])).unwrap_err().is_illegal_argument());
}

#[test]
fn more_skills_than_the_cap_are_refused() {
    let many: Vec<String> = (0..=SKILLS_PER_OWNER_MAX).map(|index| format!("skill-{index}")).collect();

    assert!(distinct_names(many).unwrap_err().is_illegal_argument());
}

#[test]
fn an_empty_list_clears_the_owner() {
    assert!(distinct_names(Vec::new()).unwrap().is_empty());
}
