use super::*;

fn origin(host: &str, path: &str) -> Option<RepositoryOrigin> {
    Some(RepositoryOrigin { host: host.into(), path: path.into() })
}

#[test]
fn https_ssh_and_the_short_form_all_name_a_host_and_a_project_path() {
    assert_eq!(RepositoryOrigin::parse("https://gitlab.example.com/acme/billing.git"), origin("gitlab.example.com", "acme/billing"));
    assert_eq!(RepositoryOrigin::parse("ssh://git@GitLab.example.com:2222/acme/sub/orders.git"), origin("gitlab.example.com", "acme/sub/orders"));
    assert_eq!(RepositoryOrigin::parse("git@gitlab.example.com:acme/billing.git"), origin("gitlab.example.com", "acme/billing"));
    assert_eq!(RepositoryOrigin::parse("https://gitlab.example.com/acme/billing/"), origin("gitlab.example.com", "acme/billing"));
}

#[test]
fn a_user_or_token_in_the_url_is_dropped() {
    let parsed = RepositoryOrigin::parse("https://oauth2:glpat-secret@gitlab.example.com/acme/billing.git").unwrap();
    assert_eq!(parsed.display(), "gitlab.example.com/acme/billing");
}

#[test]
fn a_local_path_or_another_scheme_is_no_origin() {
    assert_eq!(RepositoryOrigin::parse("/srv/git/billing.git"), None);
    assert_eq!(RepositoryOrigin::parse("../billing"), None);
    assert_eq!(RepositoryOrigin::parse("file:///srv/git/billing.git"), None);
    assert_eq!(RepositoryOrigin::parse("https://gitlab.example.com/"), None);
}

#[test]
fn an_origin_matches_the_instance_on_its_host_whatever_the_port() {
    let billing = RepositoryOrigin::parse("ssh://git@gitlab.example.com:2222/acme/billing.git").unwrap();
    assert_eq!(billing.path_on("https://gitlab.example.com").as_deref(), Some("acme/billing"));
    assert_eq!(billing.path_on("https://GITLAB.example.com:8443").as_deref(), Some("acme/billing"));
    assert_eq!(billing.path_on("https://gitlab.other.com"), None);
}

#[test]
fn an_instance_under_a_sub_path_drops_it_from_an_https_origin_only() {
    let over_https = RepositoryOrigin::parse("https://example.com/gitlab/acme/billing.git").unwrap();
    let over_ssh = RepositoryOrigin::parse("git@example.com:acme/billing.git").unwrap();
    assert_eq!(over_https.path_on("https://example.com/gitlab").as_deref(), Some("acme/billing"));
    assert_eq!(over_ssh.path_on("https://example.com/gitlab").as_deref(), Some("acme/billing"));
}
