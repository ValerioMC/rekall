use super::*;

fn subcommands(command: &str) -> Vec<String> {
    git_subcommands(command).unwrap()
}

#[test]
fn every_git_subcommand_of_a_chained_line_is_listed_in_order() {
    assert_eq!(subcommands("git add . && git commit -m 'a && b'; echo x | git push"), ["add", "commit", "push"]);
}

#[test]
fn text_inside_quotes_is_not_a_command() {
    assert!(subcommands("echo \"git commit\" 'git push'").is_empty());
}

#[test]
fn a_script_given_to_a_shell_is_read_too() {
    assert_eq!(subcommands("sh -c 'git status && git commit'"), ["status", "commit"]);
}

#[test]
fn options_before_the_subcommand_are_skipped() {
    assert_eq!(subcommands("git -C dir --no-pager -c a=b commit"), ["commit"]);
}

#[test]
fn an_unclosed_quote_is_an_error_not_a_guess() {
    assert_eq!(git_subcommands("git commit -m 'oops"), Err(UnreadableCommand::UnterminatedQuote('\'')));
    assert_eq!(git_subcommands("git commit -m \"oops"), Err(UnreadableCommand::UnterminatedQuote('"')));
}
