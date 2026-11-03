//! Reads a shell command line far enough to tell which git subcommands it runs, through `&&`,
//! pipes, subshells and `sh -c`. It is not a shell: variables and aliases are not expanded.

const SEPARATORS: [char; 7] = [';', '&', '|', '(', ')', '\n', '`'];
const SHELLS: [&str; 4] = ["sh", "bash", "zsh", "dash"];
const GIT_OPTIONS_WITH_VALUE: [&str; 5] = ["-C", "-c", "--git-dir", "--work-tree", "--namespace"];

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub(in crate::guard) enum UnreadableCommand {
    #[error("the {0} quote is never closed")]
    UnterminatedQuote(char),
}

pub(in crate::guard) fn git_subcommands(command: &str) -> Result<Vec<String>, UnreadableCommand> {
    let mut subcommands = Vec::new();
    for words in split_into_commands(command)? {
        match script_run_by_shell(&words) {
            Some(script) => subcommands.extend(git_subcommands(script)?),
            None => subcommands.extend(git_subcommand(&words)),
        }
    }
    Ok(subcommands)
}

fn script_run_by_shell(words: &[String]) -> Option<&str> {
    let first = words.first()?;
    if !SHELLS.contains(&base_name(first)) {
        return None;
    }
    let flag_position = words.iter().position(|word| is_command_flag(word))?;
    words.get(flag_position + 1).map(String::as_str)
}

fn is_command_flag(word: &str) -> bool {
    word.starts_with('-') && !word.starts_with("--") && word.contains('c')
}

fn git_subcommand(words: &[String]) -> Option<String> {
    let mut remaining = words.iter().skip_while(|word| is_assignment(word));
    if base_name(remaining.next()?) != "git" {
        return None;
    }
    while let Some(word) = remaining.next() {
        if GIT_OPTIONS_WITH_VALUE.contains(&word.as_str()) {
            remaining.next();
        } else if !word.starts_with('-') {
            return Some(word.clone());
        }
    }
    None
}

fn is_assignment(word: &str) -> bool {
    match word.split_once('=') {
        Some((name, _)) => !name.is_empty() && name.chars().all(|letter| letter.is_alphanumeric() || letter == '_'),
        None => false,
    }
}

fn base_name(word: &str) -> &str {
    word.rsplit('/').next().unwrap_or(word)
}

fn split_into_commands(command: &str) -> Result<Vec<Vec<String>>, UnreadableCommand> {
    let mut reader = WordReader::default();
    let mut characters = command.chars();
    while let Some(character) = characters.next() {
        match character {
            '\'' => reader.push_text(&single_quoted(&mut characters)?),
            '"' => reader.push_text(&double_quoted(&mut characters)?),
            '\\' => reader.push_text(&characters.next().map(String::from).unwrap_or_default()),
            separator if SEPARATORS.contains(&separator) => reader.end_command(),
            space if space.is_whitespace() => reader.end_word(),
            other => reader.push_text(&other.to_string()),
        }
    }
    Ok(reader.finish())
}

fn single_quoted(characters: &mut std::str::Chars) -> Result<String, UnreadableCommand> {
    let mut text = String::new();
    for character in characters {
        if character == '\'' {
            return Ok(text);
        }
        text.push(character);
    }
    Err(UnreadableCommand::UnterminatedQuote('\''))
}

fn double_quoted(characters: &mut std::str::Chars) -> Result<String, UnreadableCommand> {
    let mut text = String::new();
    while let Some(character) = characters.next() {
        match character {
            '"' => return Ok(text),
            '\\' => text.extend(characters.next()),
            other => text.push(other),
        }
    }
    Err(UnreadableCommand::UnterminatedQuote('"'))
}

#[derive(Default)]
struct WordReader {
    commands: Vec<Vec<String>>,
    words: Vec<String>,
    word: String,
    inside_word: bool,
}

impl WordReader {
    fn push_text(&mut self, text: &str) {
        self.word.push_str(text);
        self.inside_word = true;
    }

    fn end_word(&mut self) {
        if self.inside_word {
            self.words.push(std::mem::take(&mut self.word));
            self.inside_word = false;
        }
    }

    fn end_command(&mut self) {
        self.end_word();
        if !self.words.is_empty() {
            self.commands.push(std::mem::take(&mut self.words));
        }
    }

    fn finish(mut self) -> Vec<Vec<String>> {
        self.end_command();
        self.commands
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/guard/rule/shell_command_tests.rs"]
mod tests;
