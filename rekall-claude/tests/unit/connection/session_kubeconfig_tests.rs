use std::path::PathBuf;

use rekall_service::connection::Secret;

use super::*;

fn access(credential: KubeCredential) -> KubeAccess {
    KubeAccess {
        context: "original".into(),
        server: "https://k8s.example.com:6443".into(),
        certificate_authority: Some("Q0E=".into()),
        tls_server_name: None,
        namespace: Some("default-ns".into()),
        credential,
    }
}

#[test]
fn the_session_kubeconfig_names_the_cluster_by_label_and_holds_no_credential() {
    let prod = access(KubeCredential::Token(Secret::new("sa-token-123")));
    let text = session_kubeconfig(&[SessionCluster { label: "prod", access: &prod, namespace: Some("apps") }], &PathBuf::from("/s/t1/rekall-kube-credential"));
    let document: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(document["current-context"], "prod");
    assert_eq!(document["clusters"][0]["cluster"]["server"], "https://k8s.example.com:6443");
    assert_eq!(document["clusters"][0]["cluster"]["certificate-authority-data"], "Q0E=");
    assert_eq!(document["contexts"][0]["context"]["namespace"], "apps");
    assert_eq!(document["users"][0]["user"]["exec"]["command"], "/s/t1/rekall-kube-credential");
    assert_eq!(document["users"][0]["user"]["exec"]["args"][0], "prod");
    assert_eq!(document["users"][0]["user"]["exec"]["interactiveMode"], "Never");
    assert!(!text.contains("sa-token-123"));
}

#[test]
fn without_a_target_the_stored_namespace_is_kept() {
    let prod = access(KubeCredential::Token(Secret::new("t")));
    let text = session_kubeconfig(&[SessionCluster { label: "prod", access: &prod, namespace: None }], &PathBuf::from("/p"));
    assert!(text.contains("\"namespace\": \"default-ns\""));
}

#[test]
fn a_token_is_handed_to_kubectl_as_an_exec_credential() {
    let text = exec_credential(&access(KubeCredential::Token(Secret::new("sa-token-123")))).unwrap();
    let credential: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(credential["kind"], "ExecCredential");
    assert_eq!(credential["apiVersion"], "client.authentication.k8s.io/v1");
    assert_eq!(credential["status"]["token"], "sa-token-123");
}

#[test]
fn a_client_certificate_is_handed_over_as_pem_text() {
    let certificate = KubeCredential::ClientCertificate { certificate: Secret::new("Q0VSVA=="), key: Secret::new("S0VZ") };
    let credential: serde_json::Value = serde_json::from_str(&exec_credential(&access(certificate)).unwrap()).unwrap();
    assert_eq!(credential["status"]["clientCertificateData"], "CERT");
    assert_eq!(credential["status"]["clientKeyData"], "KEY");
}

#[test]
fn certificate_data_that_is_not_base64_is_refused() {
    let broken = KubeCredential::ClientCertificate { certificate: Secret::new("!!"), key: Secret::new("S0VZ") };
    assert_eq!(exec_credential(&access(broken)).unwrap_err(), "the stored client certificate is not base64");
}
