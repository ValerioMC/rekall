use super::*;

#[test]
fn every_tool_that_saves_text_for_a_person_ends_on_the_reader_contract() {
    for text in [PROPOSE_STEP, PROPOSE_FLOW, REPLAN, CREATE_TASK, NOTE, DEVIATION, DECISION] {
        assert!(text.ends_with(reader!()), "{}", &text[..60]);
    }
}
