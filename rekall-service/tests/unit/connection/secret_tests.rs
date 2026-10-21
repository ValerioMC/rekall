use super::*;

#[test]
fn a_secret_prints_as_stars_in_debug_and_display() {
    let secret = Secret::new("glpat-abc123");
    assert_eq!(format!("{secret:?}"), "***");
    assert_eq!(format!("{secret}"), "***");
}

#[test]
fn expose_is_the_way_to_read_it() {
    assert_eq!(Secret::new("glpat-abc123").expose(), "glpat-abc123");
}

#[test]
fn whitespace_only_is_blank() {
    assert!(Secret::new("  \n").is_blank());
    assert!(!Secret::new("x").is_blank());
}
