use super::*;

const TOKEN_CONFIG: &str = "
apiVersion: v1
kind: Config
current-context: prod
clusters:
- name: prod-cluster
  cluster:
    server: https://k8s.example.com:6443
    certificate-authority-data: Q0EtREFUQQ==
contexts:
- name: prod
  context:
    cluster: prod-cluster
    user: deployer
    namespace: apps
- name: cloud
  context:
    cluster: prod-cluster
    user: cloud-user
users:
- name: deployer
  user:
    token: sa-token-123
- name: cloud-user
  user:
    exec:
      apiVersion: client.authentication.k8s.io/v1
      command: aws
      args: [eks, get-token]
preferences: {}
";

fn kubeconfig(text: &str) -> Kubeconfig {
    Kubeconfig::parse(&Secret::new(text)).unwrap()
}

fn with_user(user: &str) -> String {
    format!(
        "clusters:\n- name: c\n  cluster:\n    server: https://k8s.example.com\ncontexts:\n- name: ctx\n  context:\n    cluster: c\n    user: u\nusers:\n- name: u\n  user:\n{user}"
    )
}

fn refusal_of(text: &str) -> String {
    kubeconfig(text).select("ctx").unwrap_err().message().to_string()
}

#[test]
fn a_token_context_resolves_to_its_server_namespace_and_token() {
    let access = kubeconfig(TOKEN_CONFIG).select("prod").unwrap();
    assert_eq!(access.server, "https://k8s.example.com:6443");
    assert_eq!(access.namespace.as_deref(), Some("apps"));
    assert_eq!(access.certificate_authority.as_deref(), Some("Q0EtREFUQQ=="));
    assert!(matches!(access.credential, KubeCredential::Token(ref token) if token.expose() == "sa-token-123"));
}

#[test]
fn every_context_is_listed_with_why_it_cannot_be_imported() {
    let contexts = kubeconfig(TOKEN_CONFIG).contexts();
    assert_eq!(contexts.len(), 2);
    assert_eq!(contexts[0].name, "prod");
    assert_eq!(contexts[0].user_kind, KubeUserKind::Token);
    assert_eq!(contexts[0].refusal, None);
    assert_eq!(contexts[1].user_kind, KubeUserKind::Exec);
    assert!(contexts[1].refusal.as_deref().is_some_and(|why| why.contains("runs a command")));
}

#[test]
fn a_context_the_file_does_not_have_is_refused_with_the_ones_it_has() {
    let error = kubeconfig(TOKEN_CONFIG).select("staging").unwrap_err();
    assert!(error.message().contains("no context 'staging'"));
    assert!(error.message().contains("prod, cloud"));
}

#[test]
fn a_client_certificate_user_resolves_to_its_certificate_and_key() {
    let access = kubeconfig(&with_user("    client-certificate-data: Q0VSVA==\n    client-key-data: S0VZ\n")).select("ctx").unwrap();
    match access.credential {
        KubeCredential::ClientCertificate { certificate, key } => {
            assert_eq!(certificate.expose(), "Q0VSVA==");
            assert_eq!(key.expose(), "S0VZ");
        }
        KubeCredential::Token(_) => panic!("a client certificate was expected"),
    }
}

#[test]
fn files_the_kubeconfig_names_are_read_and_inlined() {
    let folder = tempfile::tempdir().unwrap();
    let ca = folder.path().join("ca.crt");
    let certificate = folder.path().join("client.crt");
    let key = folder.path().join("client.key");
    let token = folder.path().join("token");
    std::fs::write(&ca, "CA").unwrap();
    std::fs::write(&certificate, "CERT").unwrap();
    std::fs::write(&key, "KEY").unwrap();
    std::fs::write(&token, "file-token\n").unwrap();
    let text = format!(
        "clusters:\n- name: c\n  cluster:\n    server: https://k8s.example.com\n    certificate-authority: {}\ncontexts:\n- name: cert\n  context: {{cluster: c, user: cert}}\n- name: token\n  context: {{cluster: c, user: token}}\nusers:\n- name: cert\n  user:\n    client-certificate: {}\n    client-key: {}\n- name: token\n  user:\n    tokenFile: {}\n",
        ca.display(), certificate.display(), key.display(), token.display()
    );
    let config = kubeconfig(&text);

    let with_certificate = config.select("cert").unwrap();
    assert_eq!(with_certificate.certificate_authority.as_deref(), Some("Q0E="));
    assert!(matches!(with_certificate.credential, KubeCredential::ClientCertificate { ref certificate, ref key } if certificate.expose() == "Q0VSVA==" && key.expose() == "S0VZ"));
    assert!(matches!(config.select("token").unwrap().credential, KubeCredential::Token(ref token) if token.expose() == "file-token"));
}

#[test]
fn users_that_are_not_the_persons_own_credential_are_refused() {
    assert!(refusal_of(&with_user("    auth-provider:\n      name: oidc\n")).contains("auth provider"));
    assert!(refusal_of(&with_user("    username: admin\n    password: secret\n")).contains("username and password"));
    assert!(refusal_of(&with_user("    {}\n")).contains("no token and no client certificate"));
}

#[test]
fn a_client_certificate_without_its_key_is_refused() {
    assert!(refusal_of(&with_user("    client-certificate-data: Q0VSVA==\n")).contains("no client key"));
}

#[test]
fn a_relative_file_path_is_refused() {
    assert!(refusal_of(&with_user("    tokenFile: token.txt\n")).contains("relative"));
}

#[test]
fn a_file_that_cannot_be_read_is_refused_with_its_path() {
    let error = refusal_of(&with_user("    tokenFile: /nonexistent/rekall/token\n"));
    assert!(error.contains("/nonexistent/rekall/token"));
}

#[test]
fn a_server_without_tls_or_skipping_verification_is_refused() {
    let plain = "clusters:\n- name: c\n  cluster:\n    server: http://k8s.example.com\ncontexts:\n- name: ctx\n  context: {cluster: c, user: u}\nusers:\n- name: u\n  user: {token: t}\n";
    let insecure = "clusters:\n- name: c\n  cluster:\n    server: https://k8s.example.com\n    insecure-skip-tls-verify: true\ncontexts:\n- name: ctx\n  context: {cluster: c, user: u}\nusers:\n- name: u\n  user: {token: t}\n";
    assert!(refusal_of(plain).contains("not an https://"));
    assert!(refusal_of(insecure).contains("skips TLS verification"));
}

fn with_proxy(proxy: &str) -> String {
    format!("clusters:\n- name: c\n  cluster:\n    server: https://10.0.0.1:6443\n    proxy-url: {proxy}\ncontexts:\n- name: ctx\n  context: {{cluster: c, user: u}}\nusers:\n- name: u\n  user: {{token: t}}\n")
}

#[test]
fn a_cluster_behind_a_proxy_keeps_its_proxy_url() {
    let access = kubeconfig(&with_proxy("http://10.250.0.1:33128")).select("ctx").unwrap();
    assert_eq!(access.proxy_url.as_deref(), Some("http://10.250.0.1:33128"));
    assert_eq!(kubeconfig(TOKEN_CONFIG).select("prod").unwrap().proxy_url, None);
}

#[test]
fn a_proxy_the_check_cannot_use_or_that_carries_a_password_is_refused() {
    assert!(refusal_of(&with_proxy("socks5://10.250.0.1:1080")).contains("not an http:// or https://"));
    let with_password = refusal_of(&with_proxy("http://admin:hunter2@10.250.0.1:33128"));
    assert!(with_password.contains("username or password"));
    assert!(!with_password.contains("hunter2"));
}

#[test]
fn a_broken_document_is_refused_without_quoting_it() {
    let error = Kubeconfig::parse(&Secret::new("users:\n- name: u\n  user:\n    token: [sa-token-xyz\n")).err().unwrap();
    assert!(error.message().contains("not a valid kubeconfig"));
    assert!(!error.message().contains("sa-token-xyz"));
}

#[test]
fn an_empty_kubeconfig_is_refused() {
    assert!(Kubeconfig::parse(&Secret::new("  ")).is_err());
}
