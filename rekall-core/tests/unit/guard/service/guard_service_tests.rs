use super::*;

const CREDENTIAL_FILE: &str = "/home/dev/.rekall/credentials.json";

fn context(auto_commit: bool) -> GuardContext {
    GuardContext {
        auto_commit,
        running_step_number: None,
        protected_credential_references: vec![CREDENTIAL_FILE.into(), "/api/terminal/".into()],
    }
}

fn shell(command: &str) -> GuardEvent {
    GuardEvent::BeforeTool { tool_name: "Bash".into(), command_or_path: command.into() }
}

fn is_blocked(verdict: &GuardVerdict) -> bool {
    matches!(verdict, GuardVerdict::Block { .. })
}

#[test]
fn a_commit_is_blocked_on_an_auto_commit_project_and_says_what_to_do() {
    let verdict = GuardService::decide(&shell("git commit -m x"), &context(true));

    let GuardVerdict::Block { reason } = verdict else { panic!("expected a block") };
    assert!(reason.contains("git commit") && reason.contains("step:N done"), "{reason}");
}

#[test]
fn a_commit_is_allowed_without_auto_commit() {
    assert_eq!(GuardService::decide(&shell("git commit -m x"), &context(false)), GuardVerdict::Allow);
}

#[test]
fn push_and_add_are_blocked_on_an_auto_commit_project() {
    for command in ["git push origin main", "git add -A", "git add src/lib.rs"] {
        assert!(is_blocked(&GuardService::decide(&shell(command), &context(true))), "{command}");
    }
}

#[test]
fn a_commit_hidden_in_a_larger_command_is_still_blocked() {
    for command in [
        "cargo test && git commit -m x",
        "git status; git commit -m x",
        "cargo fmt || git add .",
        "echo done | git commit -F -",
        "(cd rekall-core && git commit -m x)",
        "echo $(git commit -m x)",
        "sh -c 'git commit -m x'",
        "bash -lc \"cargo test && git push\"",
        "git -C ../other commit -m x",
        "git -c user.name=me commit -m x",
        "GIT_AUTHOR_NAME=me git commit -m x",
        "/usr/bin/git commit -m x",
    ] {
        assert!(is_blocked(&GuardService::decide(&shell(command), &context(true))), "{command}");
    }
}

#[test]
fn only_whole_words_count_as_a_commit() {
    for command in ["git commit-graph write", "git status", "git log --oneline", "echo git commit", "grep 'git commit' notes.md", "cargo test --package git-commit-tool"] {
        assert_eq!(GuardService::decide(&shell(command), &context(true)), GuardVerdict::Allow, "{command}");
    }
}

#[test]
fn a_command_that_cannot_be_read_is_allowed() {
    assert_eq!(GuardService::decide(&shell("git commit -m 'unfinished"), &context(true)), GuardVerdict::Allow);
}

#[test]
fn git_in_a_tool_that_is_not_the_shell_is_not_a_command() {
    let event = GuardEvent::BeforeTool { tool_name: "Read".into(), command_or_path: "git commit".into() };

    assert_eq!(GuardService::decide(&event, &context(true)), GuardVerdict::Allow);
}

#[test]
fn credential_paths_are_blocked_in_any_tool_on_any_project() {
    for auto_commit in [true, false] {
        for event in [
            shell(&format!("cat {CREDENTIAL_FILE}")),
            shell(&format!("grep token {CREDENTIAL_FILE}")),
            shell(&format!("cp {CREDENTIAL_FILE} /tmp/copy")),
            shell("curl http://127.0.0.1:3000/api/terminal/7/credentials/gitlab"),
            GuardEvent::BeforeTool { tool_name: "Read".into(), command_or_path: CREDENTIAL_FILE.into() },
        ] {
            assert!(is_blocked(&GuardService::decide(&event, &context(auto_commit))), "{event:?}");
        }
    }
}

#[test]
fn other_paths_are_not_credentials() {
    assert_eq!(GuardService::decide(&shell("cat /home/dev/.rekall/settings.json"), &context(false)), GuardVerdict::Allow);
}

#[test]
fn stopping_is_blocked_only_while_a_step_is_running_and_names_it() {
    let running = GuardContext { running_step_number: Some(4), ..context(false) };

    let GuardVerdict::Block { reason } = GuardService::decide(&GuardEvent::BeforeStop, &running) else { panic!("expected a block") };

    assert!(reason.contains("Step 4") && reason.contains("step:4 done"), "{reason}");
    assert_eq!(GuardService::decide(&GuardEvent::BeforeStop, &context(false)), GuardVerdict::Allow);
}
