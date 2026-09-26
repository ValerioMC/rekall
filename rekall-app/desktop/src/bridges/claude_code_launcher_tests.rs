use super::*;

#[test]
fn an_anchor_is_letters_digits_and_the_few_marks_an_anchor_uses() {
    assert!(is_safe_anchor("project:vega task:report-builder"));
    assert!(is_safe_anchor("  task:a.b_c-d  "));
    assert!(!is_safe_anchor("   "));
    assert!(!is_safe_anchor("task:x; rm -rf ~"));
    assert!(!is_safe_anchor("task:$(whoami)"));
    assert!(!is_safe_anchor("task:'quoted'"));
    assert!(!is_safe_anchor(&"a".repeat(301)));
    assert!(is_safe_anchor(&"a".repeat(300)));
}

#[test]
fn the_script_changes_into_the_folder_and_execs_claude_on_the_anchor() {
    assert_eq!(
        script("/Users/me/Projects/it's here", "project:vega task:x ", true, "/Users/me/.local/bin/claude"),
        "#!/bin/sh\nrm -f \"$0\"\ncd '/Users/me/Projects/it'\\''s here' || exit 1\n\
         exec '/Users/me/.local/bin/claude' --dangerously-skip-permissions '/rk project:vega task:x'\n"
    );
    assert!(!script("/p", "task:x", false, "claude").contains("--dangerously-skip-permissions"));
}

#[test]
fn claude_is_looked_for_where_it_installs_itself_first() {
    let home = tempfile_home();
    let installed = home.join(".claude/local/claude");
    std::fs::create_dir_all(installed.parent().unwrap()).unwrap();
    std::fs::write(&installed, "#!/bin/sh\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&installed, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    assert_eq!(claude_binary(&home), installed.to_string_lossy());
    std::fs::remove_dir_all(&home).unwrap();
}

fn tempfile_home() -> PathBuf {
    let home = std::env::temp_dir().join(format!("rekall-desktop-test-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&home).unwrap();
    home
}
