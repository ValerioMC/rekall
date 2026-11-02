use super::*;
use rekall_kernel::Instant;
use rekall_store::ConnectionKind;

fn gitlab(label: &str, root_path: Option<&str>, with_credential: bool) -> external_connection::Model {
    let mut connection = external_connection::Model::new(label.into(), label.into(), ConnectionKind::Gitlab, Some("https://gitlab.example.com".into()), Instant::now());
    connection.root_path = root_path.map(str::to_string);
    connection.secret_set_at = with_credential.then(Instant::now);
    connection
}

fn origin(path: &str) -> RepositoryOrigin {
    RepositoryOrigin::parse(&format!("https://gitlab.example.com/{path}.git")).unwrap()
}

fn chosen(git_hosts: &[external_connection::Model], path: &str) -> Option<String> {
    connection_at(git_hosts, &origin(path)).map(|(connection, _)| connection.label)
}

#[test]
fn the_longest_root_path_that_covers_the_repository_wins() {
    let hosts = [gitlab("whole", None, true), gitlab("acme", Some("acme"), true), gitlab("platform", Some("acme/platform"), true)];

    assert_eq!(chosen(&hosts, "acme/platform/api").as_deref(), Some("platform"));
    assert_eq!(chosen(&hosts, "acme/billing").as_deref(), Some("acme"));
    assert_eq!(chosen(&hosts, "other/tool").as_deref(), Some("whole"));
}

#[test]
fn a_root_path_covers_whole_segments_only_and_ignores_case() {
    let hosts = [gitlab("acme", Some("Acme"), true)];

    assert_eq!(chosen(&hosts, "acme/api").as_deref(), Some("acme"));
    assert_eq!(chosen(&hosts, "acme-labs/api"), None);
}

#[test]
fn a_connection_whose_root_path_does_not_cover_the_repository_is_no_candidate() {
    let hosts = [gitlab("platform", Some("acme/platform"), true)];

    assert_eq!(chosen(&hosts, "acme/billing"), None);
}

#[test]
fn between_equal_root_paths_one_with_a_credential_is_preferred() {
    let hosts = [gitlab("empty", Some("acme"), false), gitlab("usable", Some("acme"), true)];

    assert_eq!(chosen(&hosts, "acme/api").as_deref(), Some("usable"));
}

#[test]
fn without_any_root_path_an_instance_wide_connection_with_a_credential_is_preferred_over_an_empty_one() {
    let hosts = [gitlab("empty", None, false), gitlab("usable", None, true)];

    assert_eq!(chosen(&hosts, "acme/api").as_deref(), Some("usable"));
}

#[test]
fn a_narrower_connection_without_a_credential_still_wins_so_the_missing_credential_is_reported() {
    let hosts = [gitlab("whole", None, true), gitlab("acme", Some("acme"), false)];

    assert_eq!(chosen(&hosts, "acme/api").as_deref(), Some("acme"));
}

#[test]
fn the_repository_path_comes_back_relative_to_the_instance() {
    let mut hosted_under_a_path = gitlab("sub", None, true);
    hosted_under_a_path.base_url = Some("https://gitlab.example.com/git".into());
    let origin = RepositoryOrigin::parse("https://gitlab.example.com/git/acme/api.git").unwrap();

    assert_eq!(connection_at(&[hosted_under_a_path], &origin).map(|(_, path)| path).as_deref(), Some("acme/api"));
}

#[test]
fn a_connection_at_another_host_is_no_candidate() {
    let mut elsewhere = gitlab("elsewhere", None, true);
    elsewhere.base_url = Some("https://gitlab.other.com".into());

    assert_eq!(chosen(&[elsewhere], "acme/api"), None);
}
