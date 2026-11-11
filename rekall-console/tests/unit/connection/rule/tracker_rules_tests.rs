use super::*;

#[test]
fn a_gitlab_target_is_a_path_of_at_least_a_group_and_a_project() {
    assert_eq!(tracker_location_of(ConnectionKind::Gitlab, Some(" /acme/platform/app/ ")).unwrap(), "acme/platform/app");
    assert!(tracker_location_of(ConnectionKind::Gitlab, Some("app")).is_err());
    assert!(tracker_location_of(ConnectionKind::Gitlab, Some("acme//app")).is_err());
    assert!(tracker_location_of(ConnectionKind::Gitlab, None).is_err());
}

#[test]
fn a_github_target_is_an_owner_and_a_repository() {
    assert_eq!(tracker_location_of(ConnectionKind::Github, Some("acme/app")).unwrap(), "acme/app");
    assert!(tracker_location_of(ConnectionKind::Github, Some("acme/team/app")).is_err());
    assert!(tracker_location_of(ConnectionKind::Github, Some("acme")).is_err());
}

#[test]
fn a_jira_target_is_a_project_key() {
    assert_eq!(tracker_location_of(ConnectionKind::Jira, Some("eng")).unwrap(), "ENG");
    assert!(tracker_location_of(ConnectionKind::Jira, Some("not a key")).is_err());
}

#[test]
fn a_cluster_and_a_grafana_instance_are_no_task_manager() {
    assert!(tracker_location_of(ConnectionKind::Kubernetes, Some("apps")).is_err());
    assert!(tracker_location_of(ConnectionKind::Grafana, Some("apps")).is_err());
}
