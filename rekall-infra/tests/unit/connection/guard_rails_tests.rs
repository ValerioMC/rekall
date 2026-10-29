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

#[test]
fn the_settings_are_claude_code_json_with_the_deny_list() {
    let document: serde_json::Value = serde_json::from_str(&settings_document(&PathBuf::from("/s/t1"), ENDPOINT)).unwrap();
    let deny = document["permissions"]["deny"].as_array().unwrap();
    assert_eq!(deny.len(), rules().len());
    assert!(deny.iter().any(|rule| rule == "Read(//s/t1/**)"));
}
