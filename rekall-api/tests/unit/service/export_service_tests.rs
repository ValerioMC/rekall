use super::*;

#[test]
fn a_label_full_of_separators_cannot_escape_its_folder() {
    assert_eq!(safe("../../etc"), "etc");
    assert_eq!(safe("a/../b"), "a-b");
    assert_eq!(safe("   "), "untitled");
    assert_eq!(file_name("CONTEXT.md"), "CONTEXT.md");
    assert_eq!(file_name("Cluster access"), "Cluster-access.md");
}

#[test]
fn a_name_used_twice_in_one_folder_gets_a_number() {
    let mut used = HashSet::new();
    assert_eq!(unique(&mut used, "a.md".into()), "a.md");
    assert_eq!(unique(&mut used, "a.md".into()), "a-2.md");
    assert_eq!(unique(&mut used, "a.md".into()), "a-3.md");
}
