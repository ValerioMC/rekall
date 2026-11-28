use super::*;

const CREDENTIAL_FILE: &str = "/home/dev/.rekall/credentials.json";

fn context(auto_commit: bool) -> GuardContext {
    GuardContext {
        auto_commit,
        merge: None,
        running_step: None,
        late_changes: None,
        protected_credential_references: vec![CREDENTIAL_FILE.into(), "/api/terminal/".into()],
    }
}

fn shell(command: &str) -> GuardEvent {
    GuardEvent::BeforeTool { shell_command: Some(command.into()), input_texts: vec![command.into()] }
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
    let event = GuardEvent::BeforeTool { shell_command: None, input_texts: vec!["git commit".into()] };

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
            GuardEvent::BeforeTool { shell_command: None, input_texts: vec![CREDENTIAL_FILE.into()] },
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
    let running = GuardContext { running_step: Some(crate::guard::GuardRunningStep { number: 4, task_anchor: "task:VE-12".into() }), ..context(false) };

    let GuardVerdict::Block { reason } = GuardService::decide(&GuardEvent::BeforeStop, &running) else { panic!("expected a block") };

    assert!(reason.contains("Step 4") && reason.contains("/rk task:VE-12 step:4 done"), "{reason}");
    assert_eq!(GuardService::decide(&GuardEvent::BeforeStop, &context(false)), GuardVerdict::Allow);
}

#[test]
fn merging_allows_only_the_designated_branch_operations() {
    let mut context = context(true);
    context.merge = Some(crate::guard::GuardMerge { branch: "rekall/ve-12".into(), pull_request: true });

    for command in ["git add -- src/main.rs", "git commit --no-edit", "git push origin rekall/ve-12"] {
        assert_eq!(GuardService::decide(&shell(command), &context), GuardVerdict::Allow);
    }
    for command in [
        "git -C /other add -A", "cd /other && git commit --no-edit", "git push origin main",
        "git push --force origin rekall/ve-12", "git push origin +rekall/ve-12",
    ] {
        assert!(is_blocked(&GuardService::decide(&shell(command), &context)), "{command}");
    }
    context.merge.as_mut().unwrap().pull_request = false;
    assert!(is_blocked(&GuardService::decide(&shell("git push origin rekall/ve-12"), &context)));
}

fn left_behind(files: &[&str], step_to_claim: Option<u32>) -> GuardContext {
    let late_changes = crate::guard::GuardLateChanges {
        task_anchor: "task:VE-12".into(),
        files: files.iter().map(|file| file.to_string()).collect(),
        step_to_claim,
    };
    GuardContext { late_changes: Some(late_changes), ..context(true) }
}

fn stop_reason(context: &GuardContext) -> String {
    let GuardVerdict::Block { reason } = GuardService::decide(&GuardEvent::BeforeStop, context) else { panic!("expected a block") };
    reason
}

#[test]
fn stopping_with_uncommitted_files_after_the_last_claim_names_them_and_the_step_to_claim_again() {
    let reason = stop_reason(&left_behind(&[".impeccable/config.json"], Some(2)));

    assert!(reason.contains("1 uncommitted file: .impeccable/config.json."), "{reason}");
    assert!(reason.contains("/rk task:VE-12 step:2 done"), "{reason}");
    assert!(reason.contains("delete"), "{reason}");
}

#[test]
fn a_task_with_no_steps_is_told_to_write_the_wrapup_again() {
    let reason = stop_reason(&left_behind(&["a.txt", "b.txt"], None));

    assert!(reason.contains("2 uncommitted files: a.txt, b.txt."), "{reason}");
    assert!(reason.contains("/rk task:VE-12 wrapup"), "{reason}");
    assert!(!reason.contains("step:"), "{reason}");
}

#[test]
fn only_the_first_few_files_are_listed_and_the_rest_are_counted() {
    let reason = stop_reason(&left_behind(&["a", "b", "c", "d", "e", "f", "g"], Some(1)));

    assert!(reason.contains("7 uncommitted files: a, b, c, d, e and 2 more."), "{reason}");
}

#[test]
fn leftover_files_do_not_block_a_project_that_does_not_commit_on_claim_or_a_clean_worktree() {
    let without_auto_commit = GuardContext { auto_commit: false, ..left_behind(&["a.txt"], Some(1)) };

    for context in [without_auto_commit, left_behind(&[], Some(1))] {
        assert_eq!(GuardService::decide(&GuardEvent::BeforeStop, &context), GuardVerdict::Allow);
    }
}

#[test]
fn leftover_files_do_not_block_a_tool() {
    assert_eq!(GuardService::decide(&shell("rm a.txt"), &left_behind(&["a.txt"], Some(1))), GuardVerdict::Allow);
}
