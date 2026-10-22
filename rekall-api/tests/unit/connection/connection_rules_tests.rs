use super::*;

#[test]
fn a_kind_is_read_whatever_its_case() {
    assert_eq!(kind_of(Some(" gitlab ")).unwrap(), ConnectionKind::Gitlab);
    assert_eq!(kind_of(Some("KUBERNETES")).unwrap(), ConnectionKind::Kubernetes);
    assert_eq!(kind_of(Some("jira")).unwrap(), ConnectionKind::Jira);
    assert!(kind_of(Some("ssh")).is_err());
    assert!(kind_of(None).is_err());
}

#[test]
fn a_label_left_out_is_made_from_the_title() {
    assert_eq!(label_of(None, "Production Cluster").unwrap(), "production-cluster");
    assert_eq!(label_of(Some("  "), "GitLab").unwrap(), "gitlab");
    assert_eq!(label_of(Some("Prod"), "anything").unwrap(), "prod");
}

#[test]
fn a_blank_title_is_refused() {
    assert!(title_of(Some("  ")).is_err());
    assert_eq!(title_of(Some(" GitLab ")).unwrap(), "GitLab");
}

#[test]
fn a_gitlab_url_loses_its_trailing_slash_and_a_kubernetes_one_is_refused() {
    assert_eq!(base_url_of(ConnectionKind::Gitlab, Some("https://gitlab.example.com/ ")).unwrap().as_deref(), Some("https://gitlab.example.com"));
    assert!(base_url_of(ConnectionKind::Kubernetes, Some("https://k8s.example.com")).is_err());
    assert_eq!(base_url_of(ConnectionKind::Kubernetes, None).unwrap(), None);
}

#[test]
fn a_blank_target_means_none() {
    assert_eq!(target_of(Some("  ")), None);
    assert_eq!(target_of(Some(" group/app ")).as_deref(), Some("group/app"));
}

#[test]
fn a_blank_proxy_means_the_site_is_reached_direct() {
    assert_eq!(proxy_url_of(Some("  ")), None);
    assert_eq!(proxy_url_of(Some(" http://proxy.example.com:8080 ")).as_deref(), Some("http://proxy.example.com:8080"));
}
