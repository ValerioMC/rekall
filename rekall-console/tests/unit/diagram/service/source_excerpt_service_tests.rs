use std::fs;

use super::*;

fn project() -> tempfile::TempDir {
    let folder = tempfile::tempdir().unwrap();
    fs::create_dir_all(folder.path().join("src")).unwrap();
    let body: String = (1..=30).map(|n| format!("line {n}\n")).collect();
    fs::write(folder.path().join("src/order.rs"), body).unwrap();
    folder
}

#[test]
fn a_span_comes_back_with_the_lines_around_it() {
    let folder = project();
    let excerpt = read_excerpt(folder.path(), "src/order.rs", Some(10), Some(12), ExcerptScope::Around).unwrap();

    assert_eq!((excerpt.highlight_start, excerpt.highlight_end), (Some(10), Some(12)));
    assert_eq!(excerpt.lines.first().unwrap().number, 6);
    assert_eq!(excerpt.lines.last().unwrap().number, 16);
    assert_eq!(excerpt.lines[4].text, "line 10");
    assert_eq!(excerpt.language.as_deref(), Some("rust"));
    assert_eq!(excerpt.total_lines, 30);
}

#[test]
fn the_whole_file_comes_back_with_only_the_span_marked() {
    let folder = project();
    let excerpt = read_excerpt(folder.path(), "src/order.rs", Some(10), Some(12), ExcerptScope::WholeFile).unwrap();

    assert_eq!((excerpt.highlight_start, excerpt.highlight_end), (Some(10), Some(12)));
    assert_eq!(excerpt.lines.first().unwrap().number, 1);
    assert_eq!(excerpt.lines.last().unwrap().number, 30);
    assert!(!excerpt.truncated);
}

#[test]
fn a_whole_file_longer_than_the_cap_is_cut_and_flagged() {
    let folder = tempfile::tempdir().unwrap();
    let body: String = (1..=WHOLE_FILE_MAX_LINES + 10).map(|n| format!("line {n}\n")).collect();
    fs::write(folder.path().join("big.rs"), body).unwrap();

    let excerpt = read_excerpt(folder.path(), "big.rs", None, None, ExcerptScope::WholeFile).unwrap();

    assert_eq!(excerpt.lines.len() as u32, WHOLE_FILE_MAX_LINES);
    assert!(excerpt.truncated);
}

#[test]
fn a_span_past_the_end_is_clamped_to_the_file() {
    let folder = project();
    let excerpt = read_excerpt(folder.path(), "./src/order.rs", Some(29), Some(90), ExcerptScope::Around).unwrap();
    assert_eq!((excerpt.highlight_start, excerpt.highlight_end), (Some(29), Some(30)));
    assert_eq!(excerpt.lines.last().unwrap().number, 30);
}

#[test]
fn a_path_climbing_out_of_the_project_folder_is_refused() {
    let folder = project();
    let outside = tempfile::NamedTempFile::new().unwrap();
    let climbing = format!("../{}", outside.path().file_name().unwrap().to_string_lossy());
    let moved = folder.path().join("src");

    assert!(matches!(read_excerpt(&moved, &climbing, None, None, ExcerptScope::Around), Err(RekallError::IllegalArgument(_)) | Err(RekallError::NotFound(_))));
    assert!(matches!(read_excerpt(folder.path(), &outside.path().to_string_lossy(), None, None, ExcerptScope::Around), Err(RekallError::IllegalArgument(_))));
}

#[test]
fn a_missing_file_is_not_found() {
    let folder = project();
    assert!(matches!(read_excerpt(folder.path(), "src/nope.rs", Some(1), None, ExcerptScope::Around), Err(RekallError::NotFound(_))));
}
