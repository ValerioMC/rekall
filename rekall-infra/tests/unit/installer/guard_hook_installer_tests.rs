use serde_json::Value;

use super::*;

struct Harness {
    home: tempfile::TempDir,
}

impl Harness {
    fn new() -> Self {
        Self { home: tempfile::tempdir().unwrap() }
    }

    fn installer(&self) -> GuardHookInstaller {
        GuardHookInstaller::new(self.home.path(), PathBuf::from("/Apps/Rekall/rekall-server"))
    }

    fn settings_path(&self) -> PathBuf {
        self.home.path().join(".claude/settings.json")
    }

    fn write(&self, text: &str) {
        fs::create_dir_all(self.home.path().join(".claude")).unwrap();
        fs::write(self.settings_path(), text).unwrap();
    }

    fn settings(&self) -> Value {
        serde_json::from_str(&fs::read_to_string(self.settings_path()).unwrap()).unwrap()
    }
}

const USER_HOOK: &str = r#"{"matcher":"Bash","hooks":[{"type":"command","command":"/usr/local/bin/my-audit"}]}"#;

fn user_settings() -> String {
    format!(r#"{{"model":"opus","hooks":{{"PreToolUse":[{USER_HOOK}],"Notification":[{{"hooks":[{{"type":"command","command":"say done"}}]}}]}},"permissions":{{"allow":["Bash(ls)"]}}}}"#)
}

#[test]
fn a_missing_file_is_created_with_only_rekalls_hooks() {
    let harness = Harness::new();

    let protection = harness.installer().install().unwrap();

    assert_eq!(protection.state, GuardProtectionState::Installed);
    let settings = harness.settings();
    assert_eq!(settings["hooks"]["PreToolUse"][0]["matcher"], "Bash|Read|Edit|Write");
    assert_eq!(settings["hooks"]["PreToolUse"][0]["hooks"][0]["command"], "'/Apps/Rekall/rekall-server' guard before-tool");
    assert_eq!(settings["hooks"]["Stop"][0]["hooks"][0]["command"], "'/Apps/Rekall/rekall-server' guard before-stop");
    assert_eq!(settings.as_object().unwrap().len(), 1);
}

#[test]
fn the_users_own_hooks_and_keys_survive_an_install_and_a_removal() {
    let harness = Harness::new();
    harness.write(&user_settings());
    let before = harness.settings();

    harness.installer().install().unwrap();
    let installed = harness.settings();
    assert_eq!(installed["hooks"]["PreToolUse"][0], before["hooks"]["PreToolUse"][0]);
    assert_eq!(installed["hooks"]["PreToolUse"].as_array().unwrap().len(), 2);
    assert_eq!(installed["model"], "opus");
    assert_eq!(installed["permissions"], before["permissions"]);

    let protection = harness.installer().uninstall().unwrap();

    assert_eq!(protection.state, GuardProtectionState::NotInstalled);
    assert_eq!(harness.settings(), before);
}

#[test]
fn installing_twice_leaves_one_entry_each() {
    let harness = Harness::new();

    harness.installer().install().unwrap();
    harness.installer().install().unwrap();

    let settings = harness.settings();
    assert_eq!(settings["hooks"]["PreToolUse"].as_array().unwrap().len(), 1);
    assert_eq!(settings["hooks"]["Stop"].as_array().unwrap().len(), 1);
}

#[test]
fn an_install_from_another_location_replaces_the_old_path() {
    let harness = Harness::new();
    GuardHookInstaller::new(harness.home.path(), PathBuf::from("/old/rekall-server")).install().unwrap();
    assert_eq!(harness.installer().status().state, GuardProtectionState::NotInstalled);

    harness.installer().install().unwrap();

    assert_eq!(harness.settings()["hooks"]["Stop"].as_array().unwrap().len(), 1);
    assert_eq!(harness.installer().status().state, GuardProtectionState::Installed);
}

#[test]
fn a_file_that_is_not_json_is_refused_and_left_as_it_was() {
    let harness = Harness::new();
    harness.write("{ \"model\": ");

    let refusal = harness.installer().install().unwrap_err();

    assert!(refusal.is_illegal_argument());
    assert_eq!(fs::read_to_string(harness.settings_path()).unwrap(), "{ \"model\": ");
    assert_eq!(harness.installer().status().state, GuardProtectionState::InvalidSettings);
    assert!(harness.installer().uninstall().unwrap_err().is_illegal_argument());
}

#[test]
fn hooks_that_are_not_the_expected_shape_are_refused_and_left_as_they_were() {
    let harness = Harness::new();
    harness.write(r#"{"hooks":{"Stop":"nope"}}"#);

    assert!(harness.installer().install().unwrap_err().is_illegal_argument());
    assert_eq!(fs::read_to_string(harness.settings_path()).unwrap(), r#"{"hooks":{"Stop":"nope"}}"#);
}

#[test]
fn the_first_write_keeps_a_copy_and_a_later_one_does_not_replace_it() {
    let harness = Harness::new();
    harness.write(&user_settings());
    let backup = PathBuf::from(format!("{}.rekall-backup", harness.settings_path().display()));

    harness.installer().install().unwrap();
    harness.installer().uninstall().unwrap();
    harness.installer().install().unwrap();

    assert_eq!(fs::read_to_string(backup).unwrap(), user_settings());
}

#[test]
fn removing_from_a_missing_file_creates_nothing() {
    let harness = Harness::new();

    let protection = harness.installer().uninstall().unwrap();

    assert_eq!(protection.state, GuardProtectionState::NotInstalled);
    assert!(!harness.settings_path().exists());
}

#[test]
fn a_path_with_a_quote_is_quoted_for_the_shell() {
    assert_eq!(shell_quoted("/Users/o'neil/rekall"), "'/Users/o'\\''neil/rekall'");
}
