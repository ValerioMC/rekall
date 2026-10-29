use super::*;

const TWO_CONTEXTS: &str = "
clusters:
- name: prod-cluster
  cluster:
    server: https://k8s.example.com:6443
    certificate-authority-data: Q0E=
    proxy-url: http://proxy.example.com:3128
- name: other-cluster
  cluster:
    server: https://other.example.com
contexts:
- name: prod
  context: {cluster: prod-cluster, user: deployer, namespace: apps}
- name: other
  context: {cluster: other-cluster, user: other-user}
users:
- name: deployer
  user: {token: sa-token-123}
- name: other-user
  user: {token: other-token}
";

#[test]
fn the_stored_document_reads_back_as_the_same_access() {
    let access = Kubeconfig::parse(&Secret::new(TWO_CONTEXTS)).unwrap().select("prod").unwrap();
    let back = KubeAccess::from_stored(&access.to_kubeconfig().unwrap()).unwrap();
    assert_eq!(back.context, "prod");
    assert_eq!(back.server, "https://k8s.example.com:6443");
    assert_eq!(back.certificate_authority.as_deref(), Some("Q0E="));
    assert_eq!(back.proxy_url.as_deref(), Some("http://proxy.example.com:3128"));
    assert_eq!(back.namespace.as_deref(), Some("apps"));
    assert!(matches!(back.credential, KubeCredential::Token(ref token) if token.expose() == "sa-token-123"));
}

#[test]
fn the_stored_document_keeps_the_chosen_context_alone() {
    let access = Kubeconfig::parse(&Secret::new(TWO_CONTEXTS)).unwrap().select("prod").unwrap();
    let stored = access.to_kubeconfig().unwrap();
    assert!(!stored.expose().contains("other"));
    assert!(stored.expose().contains("current-context: prod"));
}

#[test]
fn a_stored_document_that_no_longer_reads_is_an_internal_failure() {
    let error = KubeAccess::from_stored(&Secret::new("clusters: []\n")).unwrap_err();
    assert!(matches!(error, RekallError::Internal(_)));
}
