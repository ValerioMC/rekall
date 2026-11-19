use super::*;

#[test]
fn the_answer_says_the_old_text_is_gone_when_the_description_changed() {
    let replaced = DescriptionReplaced { task_anchor: "project:vega task:pay".into(), changed: true };

    assert_eq!(report(&replaced), "Description of `project:vega task:pay` replaced. The earlier text is not kept anywhere.");
}

#[test]
fn the_answer_says_nothing_changed_when_the_text_was_the_same() {
    let replaced = DescriptionReplaced { task_anchor: "project:vega task:pay".into(), changed: false };

    assert_eq!(report(&replaced), "Description of `project:vega task:pay` already read exactly this: nothing changed.");
}
