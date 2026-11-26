use serde_json::json;

use super::*;

#[test]
fn codex_status_uses_the_clis_trust_and_enabled_flags_for_both_rekall_hooks() {
    let path = PathBuf::from("/home/dev/.codex/hooks.json");
    let file = GuardHookFile::new(path.clone(), PathBuf::from("/usr/local/bin/rekall-server"), SessionProvider::Codex);
    let [tool, stop] = file.commands();

    for (enabled, trust, expected) in [
        (true, "trusted", GuardProtectionState::Installed),
        (true, "untrusted", GuardProtectionState::TrustRequired),
        (true, "modified", GuardProtectionState::TrustRequired),
        (false, "trusted", GuardProtectionState::Disabled),
    ] {
        let report = serde_json::from_value(json!({
            "data": [{ "hooks": [
                { "command": tool, "sourcePath": path, "enabled": true, "trustStatus": "trusted" },
                { "command": stop, "sourcePath": path, "enabled": enabled, "trustStatus": trust }
            ] }]
        }))
        .unwrap();
        let protection = codex_protection(&file, &report, &path);
        assert_eq!(protection.state, expected);
        assert!(protection.configured);
    }
}

#[test]
fn a_codex_hook_from_an_unrelated_source_does_not_count_as_rekalls() {
    let path = PathBuf::from("/home/dev/.codex/hooks.json");
    let file = GuardHookFile::new(path.clone(), PathBuf::from("/usr/local/bin/rekall-server"), SessionProvider::Codex);
    let [tool, stop] = file.commands();
    let report = serde_json::from_value(json!({
        "data": [{ "hooks": [
            { "command": tool, "sourcePath": "/work/vega/.codex/hooks.json", "enabled": true, "trustStatus": "trusted" },
            { "command": stop, "sourcePath": path, "enabled": true, "trustStatus": "trusted" }
        ] }]
    }))
    .unwrap();

    assert_eq!(codex_protection(&file, &report, &path).state, GuardProtectionState::Disabled);
}
