use super::*;

#[test]
fn github_dot_com_is_answered_by_its_api_host() {
    assert_eq!(github_api_root("https://github.com/"), "https://api.github.com");
}

#[test]
fn an_enterprise_server_answers_under_api_v3() {
    assert_eq!(github_api_root("https://git.example.com"), "https://git.example.com/api/v3");
}
