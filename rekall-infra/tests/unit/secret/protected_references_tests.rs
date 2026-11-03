use super::*;

#[test]
fn both_credential_files_and_the_helper_endpoint_are_protected() {
    let references = protected_references(Path::new("/home/dev/.rekall"), Path::new("/home/dev"));

    assert_eq!(references, ["/home/dev/.rekall/secrets.json", "/home/dev/.claude/.credentials.json", "/api/terminal/"]);
}
