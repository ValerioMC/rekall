use std::path::Path;

use super::*;

#[test]
fn a_path_prints_the_way_java_printed_it() {
    assert_eq!(display_path(Path::new("/repo//vega/")), "/repo/vega");
    assert_eq!(display_path(Path::new("/")), "/");
}
