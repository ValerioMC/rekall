use super::*;

#[test]
fn github_com_takes_attachments_on_its_upload_host() {
    assert_eq!(github_upload_root("https://github.com/"), Some("https://uploads.github.com"));
}

#[test]
fn an_enterprise_server_takes_none() {
    assert_eq!(github_upload_root("https://github.example.com"), None);
}
