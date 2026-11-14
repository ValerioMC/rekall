use super::*;

const MODIFIED: &str = "\
diff --git a/src/lib.rs b/src/lib.rs
index 1111111..2222222 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -3,3 +3,4 @@ fn open() {
 let a = 1;
-let b = 2;
+let b = 3;
+let c = 4;
 let d = 5;
@@ -20,2 +21,2 @@
 tail
-old
+new
";

#[test]
fn a_modified_file_has_its_hunks_with_old_and_new_line_numbers() {
    let files = UnifiedDiff::parse(MODIFIED);

    assert_eq!(files.len(), 1);
    let file = &files[0];
    assert_eq!((file.path.as_str(), file.kind, file.additions, file.deletions), ("src/lib.rs", ChangeKind::Modified, 3, 2));
    assert_eq!(file.hunks.len(), 2);
    let first = &file.hunks[0];
    assert_eq!((first.old_start, first.new_start, first.context.as_str()), (3, 3, "fn open() {"));
    let numbers: Vec<_> = first.lines.iter().map(|l| (l.kind, l.old_number, l.new_number)).collect();
    assert_eq!(
        numbers,
        vec![
            (DiffLineKind::Context, Some(3), Some(3)),
            (DiffLineKind::Removed, Some(4), None),
            (DiffLineKind::Added, None, Some(4)),
            (DiffLineKind::Added, None, Some(5)),
            (DiffLineKind::Context, Some(5), Some(6)),
        ]
    );
}

#[test]
fn a_removed_line_that_looks_like_a_file_header_stays_inside_its_hunk() {
    let patch = "diff --git a/a.md b/a.md\n--- a/a.md\n+++ b/a.md\n@@ -1,2 +1,1 @@\n--- a/not-a-header\n keep\n";

    let files = UnifiedDiff::parse(patch);

    assert_eq!(files.len(), 1);
    assert_eq!(files[0].deletions, 1);
    assert_eq!(files[0].hunks[0].lines[0].text, "-- a/not-a-header");
}

#[test]
fn a_new_file_a_deleted_file_and_a_rename_carry_their_kind_and_paths() {
    let patch = "\
diff --git a/new.txt b/new.txt
new file mode 100644
--- /dev/null
+++ b/new.txt
@@ -0,0 +1 @@
+hello
diff --git a/gone.txt b/gone.txt
deleted file mode 100644
--- a/gone.txt
+++ /dev/null
@@ -1 +0,0 @@
-bye
diff --git a/old name.txt b/moved.txt
similarity index 100%
rename from old name.txt
rename to moved.txt
";

    let files = UnifiedDiff::parse(patch);

    let summary: Vec<_> = files.iter().map(|f| (f.path.as_str(), f.old_path.as_deref(), f.kind)).collect();
    assert_eq!(
        summary,
        vec![
            ("new.txt", None, ChangeKind::Added),
            ("gone.txt", None, ChangeKind::Deleted),
            ("moved.txt", Some("old name.txt"), ChangeKind::Renamed),
        ]
    );
}

#[test]
fn a_binary_file_has_no_hunks() {
    let patch = "diff --git a/logo.png b/logo.png\nnew file mode 100644\nBinary files /dev/null and b/logo.png differ\n";

    let files = UnifiedDiff::parse(patch);

    assert!(files[0].binary);
    assert!(files[0].hunks.is_empty());
}

#[test]
fn a_patch_cut_in_the_middle_of_a_hunk_keeps_the_lines_read() {
    let patch = "diff --git a/a.txt b/a.txt\n--- a/a.txt\n+++ b/a.txt\n@@ -1,5 +1,5 @@\n one\n-two\n+TWO\n";

    let files = UnifiedDiff::parse(patch);

    assert_eq!(files[0].hunks[0].lines.len(), 3);
}

#[test]
fn the_no_newline_marker_is_not_a_line() {
    let patch = "diff --git a/a.txt b/a.txt\n--- a/a.txt\n+++ b/a.txt\n@@ -1 +1 @@\n-a\n\\ No newline at end of file\n+b\n\\ No newline at end of file\n";

    let files = UnifiedDiff::parse(patch);

    assert_eq!(files[0].hunks[0].lines.len(), 2);
}

#[test]
fn an_empty_patch_has_no_files() {
    assert!(UnifiedDiff::parse("").is_empty());
}
