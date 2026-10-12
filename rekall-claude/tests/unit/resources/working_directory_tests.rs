use super::*;

fn session(folder: &Path, name: &str, lines: &[&str]) {
    std::fs::create_dir_all(folder).unwrap();
    std::fs::write(folder.join(name), lines.join("\n")).unwrap();
}

#[test]
fn the_real_path_comes_from_a_session_that_started_there() {
    let root = tempfile::tempdir().unwrap();
    let folder = root.path().join("-Users-me-Projects-ESA--kmasters");
    session(&folder, "s1.jsonl", &[r#"{"type":"queue-operation"}"#, r#"{"cwd":"/Users/me/Projects/ESA/_kmasters","type":"user"}"#]);

    let found = WorkingDirectory::recover(&folder, "-Users-me-Projects-ESA--kmasters");

    assert_eq!(found, WorkingDirectory { path: "/Users/me/Projects/ESA/_kmasters".into(), resolved: true });
}

#[test]
fn a_cwd_that_does_not_flatten_to_the_folder_is_where_the_session_moved_to() {
    let root = tempfile::tempdir().unwrap();
    let folder = root.path().join("-work-vega");
    session(&folder, "s1.jsonl", &[r#"{"cwd":"/work/vega/api"}"#, r#"{"cwd":"/work/vega"}"#]);

    assert_eq!(WorkingDirectory::recover(&folder, "-work-vega").path, "/work/vega");
}

#[test]
fn without_a_transcript_a_folder_that_exists_is_found_by_its_flattened_name() {
    let root = tempfile::tempdir().unwrap();
    let real = root.path().join("my-project").join("api v2").join(".hidden");
    std::fs::create_dir_all(&real).unwrap();
    std::fs::create_dir_all(root.path().join("my").join("other")).unwrap();
    let name = folder_name(&real);
    let project = root.path().join("claude-side").join(&name);
    std::fs::create_dir_all(&project).unwrap();

    let found = WorkingDirectory::recover(&project, &name);

    assert_eq!(found, WorkingDirectory { path: real.to_string_lossy().into_owned(), resolved: true });
}

#[test]
fn a_name_no_folder_flattens_to_is_a_guess() {
    let root = tempfile::tempdir().unwrap();
    let project = root.path().join("-no-such-place-here-at-all");
    std::fs::create_dir_all(&project).unwrap();

    let found = WorkingDirectory::recover(&project, "-no-such-place-here-at-all");

    assert!(!found.resolved);
}

#[test]
fn with_no_session_the_path_is_a_guess_from_the_name() {
    let root = tempfile::tempdir().unwrap();
    let folder = root.path().join("-work-vega");
    std::fs::create_dir_all(&folder).unwrap();

    assert_eq!(WorkingDirectory::recover(&folder, "-work-vega"), WorkingDirectory { path: "/work/vega".into(), resolved: false });
}

#[test]
fn a_line_that_is_not_json_is_skipped() {
    let root = tempfile::tempdir().unwrap();
    let folder = root.path().join("-work-vega");
    session(&folder, "s1.jsonl", &[r#"not json "cwd""#, r#"{"cwd":"/work/vega"}"#]);

    assert!(WorkingDirectory::recover(&folder, "-work-vega").resolved);
}
