use super::ContextFill;

fn fill(used_tokens: u64, window_tokens: u64) -> ContextFill {
    ContextFill { used_tokens, window_tokens }
}

#[test]
fn the_share_is_rounded_to_a_whole_percent() {
    assert_eq!(fill(38_009, 1_000_000).percent(), 4);
    assert_eq!(fill(124, 1000).percent(), 12);
    assert_eq!(fill(125, 1000).percent(), 13);
    assert_eq!(fill(0, 1000).percent(), 0);
}

#[test]
fn a_fill_beyond_the_window_is_capped_at_one_hundred() {
    assert_eq!(fill(1_200_000, 1_000_000).percent(), 100);
}

#[test]
fn an_unknown_window_is_zero_percent() {
    assert_eq!(fill(500, 0).percent(), 0);
}

#[test]
fn a_fill_is_written_with_camel_case_fields_and_no_percent() {
    assert_eq!(serde_json::to_value(fill(5, 10)).unwrap(), serde_json::json!({"usedTokens": 5, "windowTokens": 10}));
}
