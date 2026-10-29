use super::*;

fn write(path: &Path, bytes: &[u8]) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

#[test]
fn a_folder_is_named_the_way_claude_code_names_it() {
    assert_eq!(folder_name(Path::new("/Users/me/Projects/rekall-rust/")), "-Users-me-Projects-rekall-rust");
    assert_eq!(folder_name(Path::new("/work/ESA/.kmasters")), "-work-ESA--kmasters");
}

#[test]
fn the_sessions_of_a_folder_and_its_subfolders_come_with_their_subagents_and_size() {
    let root = tempfile::tempdir().unwrap();
    let project = root.path().join("-work-vega");
    write(&project.join("s1.jsonl"), b"12345");
    write(&project.join("s1/subagents/agent-a.jsonl"), b"123");
    write(&project.join("s1/tool-results/x.json"), b"ignored");
    write(&project.join("notes.txt"), b"ignored");
    write(&root.path().join("-work-vega-api/s2.jsonl"), b"1");
    write(&root.path().join("-work-vegas/s3.jsonl"), b"1");

    let sessions = TranscriptFolder::new(root.path().to_path_buf()).sessions_of(Path::new("/work/vega")).unwrap();

    assert_eq!(sessions.iter().map(|s| s.session_id.as_str()).collect::<Vec<_>>(), ["s1", "s2"]);
    assert_eq!(sessions[0].bytes, 8);
    assert_eq!(sessions[0].subagents, [project.join("s1/subagents/agent-a.jsonl")]);
    assert!(sessions[1].subagents.is_empty());
}

#[test]
fn a_root_that_is_not_there_yet_holds_no_sessions() {
    let root = tempfile::tempdir().unwrap();
    let folder = TranscriptFolder::new(root.path().join("projects"));
    assert!(folder.sessions_of(Path::new("/work/vega")).unwrap().is_empty());
}
