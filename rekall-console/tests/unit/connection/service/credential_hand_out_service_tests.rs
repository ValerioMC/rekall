use async_trait::async_trait;
use rekall_core::connection::{Refusal, Secret};

use super::*;

/// Hands `prod`'s kubectl credential to the token `right` and refuses everything else.
struct FakeDesk;

#[async_trait]
impl CredentialDesk for FakeDesk {
    async fn hand_out(&self, _terminal_id: Id, presented: Option<&str>, label: &str, purpose: CredentialPurpose) -> HandOut {
        match (presented, label, purpose) {
            (Some("right"), "prod", CredentialPurpose::Kubectl) => HandOut::Granted { body: Secret::new("sa-token"), content_type: "application/json" },
            (Some("right"), _, _) => HandOut::refused(Refusal::Missing, "No such connection."),
            _ => HandOut::refused(Refusal::Forbidden, "This token is not this terminal's."),
        }
    }
}

#[tokio::test]
async fn the_desks_answer_comes_back_as_it_was_given() {
    let service = CredentialHandOutService::new(Arc::new(FakeDesk));
    let terminal = Id::random();

    let granted = service.hand_out(terminal, Some("right"), "prod", CredentialPurpose::Kubectl).await;
    let forged = service.hand_out(terminal, Some("guess"), "prod", CredentialPurpose::Kubectl).await;

    assert!(matches!(granted, HandOut::Granted { ref body, .. } if body.expose() == "sa-token"));
    assert!(matches!(forged, HandOut::Refused { refusal: Refusal::Forbidden, .. }));
}
