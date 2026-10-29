use super::*;

#[test]
fn the_front_matter_stopwords_and_short_words_are_left_out() {
    let print = TextFingerprint::of("---\nname: hidden\n---\n\nThe Makefile targets are phony, and an id is 42.");
    let mut terms: Vec<&str> = print.terms().collect();
    terms.sort();
    assert_eq!(terms, ["makefile", "phony", "target"]);
}

#[test]
fn a_text_is_fully_covered_by_one_that_contains_it() {
    let small = TextFingerprint::of("phony targets documented help");
    let large = TextFingerprint::of("Makefile: phony targets documented help, plus more.");
    assert_eq!(small.term_coverage_by(&large), 1.0);
    assert_eq!(small.pair_coverage_by(&large), 1.0);
}

#[test]
fn shared_vocabulary_in_another_order_does_not_cover_the_phrasing() {
    let memory = TextFingerprint::of("phony targets documented help output");
    let document = TextFingerprint::of("output help documented targets phony");
    assert_eq!(memory.term_coverage_by(&document), 1.0);
    assert_eq!(memory.pair_coverage_by(&document), 0.0);
}

#[test]
fn empty_texts_are_not_similar_to_anything() {
    let empty = TextFingerprint::of("");
    assert_eq!(empty.similarity(&TextFingerprint::of("anything meaningful here")), 0.0);
    assert_eq!(empty.term_coverage_by(&empty), 0.0);
}

#[test]
fn plurals_match_their_singular_without_breaking_words_that_end_in_s() {
    let print = TextFingerprint::of("dashes matches prometheus status targets");
    let mut terms: Vec<&str> = print.terms().collect();
    terms.sort();
    assert_eq!(terms, ["dash", "match", "prometheus", "status", "target"]);
}

#[test]
fn a_short_text_inside_a_long_one_overlaps_fully() {
    let short = TextFingerprint::of("phony targets documented help");
    let long = TextFingerprint::of("phony targets documented help plus many other unrelated subjects discussed at length");
    assert_eq!(short.overlap(&long), 1.0);
    assert!(short.similarity(&long) < 0.5);
}
