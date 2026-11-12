use super::*;

#[test]
fn a_user_and_token_in_a_remote_url_are_taken_out_of_a_message() {
    let message = "fatal: unable to access 'https://valerio:ghp_secret@github.com/acme/vega.git/': The requested URL returned error: 403";

    assert_eq!(without_credentials(message), "fatal: unable to access 'https://github.com/acme/vega.git/': The requested URL returned error: 403");
}

#[test]
fn a_message_without_a_credential_is_left_as_it_was() {
    for message in ["To https://github.com/acme/vega.git", "! [rejected] rekall/report -> rekall/report (non-fast-forward)", "git@github.com: Permission denied (publickey)."] {
        assert_eq!(without_credentials(message), message);
    }
}

#[test]
fn an_at_sign_after_the_host_is_not_a_credential() {
    assert_eq!(without_credentials("see https://github.com/acme/vega@v1 and https://u:p@host/x"), "see https://github.com/acme/vega@v1 and https://host/x");
}
