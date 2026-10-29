use crate::connection::Secret;

/// How a kubeconfig user proves who it is, in the kubeconfig's own encoding (base64 of the PEM for
/// the certificate and key). The only two forms Rekall can hand to kubectl itself.
#[derive(Debug)]
pub enum KubeCredential {
    Token(Secret),
    ClientCertificate { certificate: Secret, key: Secret },
}
