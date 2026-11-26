use std::path::PathBuf;

use super::*;

const ENDPOINT: &str = "http://127.0.0.1:47355/api/terminal/t1/credentials";

fn rules() -> Vec<String> {
    deny_rules(&PathBuf::from("/home/me/.rekall/terminal-credentials/t1"), ENDPOINT)
}

#[test]
fn the_session_folder_is_closed_to_the_file_tools_by_an_absolute_rule() {
    let rules = rules();
    assert!(rules.contains(&"Read(//home/me/.rekall/terminal-credentials/t1/**)".to_string()));
    assert!(rules.contains(&"Edit(//home/me/.rekall/terminal-credentials/t1/**)".to_string()));
}

#[test]
fn dumping_the_environment_and_calling_a_helper_by_hand_are_denied() {
    let rules = rules();
    for rule in [
        "Bash(printenv:*)",
        "Bash(env)",
        "Bash(git credential:*)",
        "Bash(kubectl config view --raw:*)",
        "Bash(rekall-kube-credential:*)",
        "Bash(/home/me/.rekall/terminal-credentials/t1/rekall-git-credential:*)",
        "Bash(curl http://127.0.0.1:47355/api/terminal/t1/credentials:*)",
        "Bash(security find-generic-password:*)",
    ] {
        assert!(rules.contains(&rule.to_string()), "{rule} is missing");
    }
}

fn permissions(allow: Vec<String>, deny: Vec<String>) -> serde_json::Value {
    let document: serde_json::Value = serde_json::from_str(&settings_document(allow, deny)).unwrap();
    document["permissions"].clone()
}

#[test]
fn the_settings_are_claude_code_json_with_the_deny_list_and_no_empty_allow_list() {
    let permissions = permissions(Vec::new(), deny_rules(&PathBuf::from("/s/t1"), ENDPOINT));

    let deny = permissions["deny"].as_array().unwrap();
    assert_eq!(deny.len(), rules().len());
    assert!(deny.iter().any(|rule| rule == "Read(//s/t1/**)"));
    assert!(permissions.get("allow").is_none());
}

#[test]
fn the_pictures_are_allowed_to_be_read_and_never_edited() {
    let allow = picture_read_rules(Some(&PathBuf::from("/Users/me/Library/App Data (old)/images")));

    assert_eq!(allow, ["Read(//Users/me/Library/App Data (old)/images/**)"]);
    let permissions = permissions(allow.clone(), Vec::new());
    assert_eq!(permissions["allow"], serde_json::json!(allow));
    assert!(permissions.get("deny").is_none());
}

#[test]
fn a_folder_name_that_looks_like_a_pattern_is_matched_as_written() {
    let allow = picture_read_rules(Some(&PathBuf::from("/data/[work]/a*b?/images")));

    assert_eq!(allow, [r"Read(//data/\[work\]/a\*b\?/images/**)"]);
}

#[test]
fn without_a_pictures_folder_nothing_is_allowed() {
    assert!(picture_read_rules(None).is_empty());
}
