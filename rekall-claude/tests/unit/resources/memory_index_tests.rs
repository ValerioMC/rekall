use super::*;

#[test]
fn a_new_memory_is_appended_with_its_hook() {
    let index = "- [A](a.md) — first\n";
    assert_eq!(with_entry(index, "b.md", "B", "second"), "- [A](a.md) — first\n- [B](b.md) — second\n");
}

#[test]
fn a_memory_without_a_description_is_listed_without_a_hook() {
    assert_eq!(with_entry("", "b.md", "B", ""), "- [B](b.md)\n");
}

#[test]
fn a_memory_already_linked_is_not_listed_twice() {
    let index = "- [A](a.md) — first\n";
    assert_eq!(with_entry(index, "a.md", "A", "again"), index);
}

#[test]
fn an_index_missing_its_final_newline_gets_one_line_per_entry() {
    assert_eq!(with_entry("- [A](a.md)", "b.md", "B", ""), "- [A](a.md)\n- [B](b.md)\n");
}

#[test]
fn removing_a_memory_drops_only_its_line() {
    let index = "# Index\n- [A](a.md) — first\n- [B](b.md) — second\n";
    assert_eq!(without_entry(index, "a.md"), "# Index\n- [B](b.md) — second\n");
}

#[test]
fn removing_the_last_line_leaves_an_empty_index() {
    assert_eq!(without_entry("- [A](a.md)\n", "a.md"), "");
}

#[test]
fn removing_a_memory_nobody_links_changes_nothing() {
    let index = "- [A](a.md)";
    assert_eq!(without_entry(index, "b.md"), index);
}

#[test]
fn a_file_whose_name_ends_another_is_not_confused_with_it() {
    let index = "- [A](xa.md)\n- [B](a.md)\n";
    assert_eq!(without_entry(index, "a.md"), "- [A](xa.md)\n");
}
