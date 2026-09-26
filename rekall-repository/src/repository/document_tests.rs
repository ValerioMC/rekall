use super::like_matches;

#[test]
fn an_unescaped_like_reads_percent_and_underscore_as_wildcards() {
    assert!(like_matches("Cluster Access", "cluster"));
    assert!(like_matches("a1b", "a_b"));
    assert!(like_matches("anything", "a%g"));
    assert!(!like_matches("abc", "abd"));
}
