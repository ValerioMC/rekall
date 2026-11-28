use super::*;

fn files(count: usize) -> Vec<String> {
    (1..=count).map(|number| format!("{number}.txt")).collect()
}

#[test]
fn a_single_file_is_named_and_counted() {
    assert_eq!(named_uncommitted_files(&files(1)), "1 uncommitted file: 1.txt");
}

#[test]
fn past_five_files_the_rest_are_counted() {
    assert_eq!(named_uncommitted_files(&files(7)), "7 uncommitted files: 1.txt, 2.txt, 3.txt, 4.txt, 5.txt and 2 more");
}
