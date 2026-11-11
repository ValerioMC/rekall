use super::*;

/// 40% of the 8,271 words the command had before it was split by mode.
const BASE_COMMAND_WORD_LIMIT: usize = 3308;

const MODE_HEADINGS: [&str; 13] = [
    "Loading",
    "Stepping",
    "Noting",
    "Sectioning",
    "Planning",
    "Replanning",
    "Rewriting",
    "Generating",
    "Explaining",
    "Inventorying",
    "Autoconfiguring",
    "Assessing",
    "Wrapping up",
];

fn files_named_in_the_mode_table() -> Vec<String> {
    PACKAGED_COMMAND
        .lines()
        .filter(|row| row.starts_with("| `"))
        .flat_map(|row| row.split('`').skip(1).step_by(2).map(str::to_string).collect::<Vec<_>>())
        .filter(|cell| cell.ends_with(".md"))
        .collect()
}

#[test]
fn every_file_the_mode_table_names_is_packaged() {
    let named = files_named_in_the_mode_table();
    assert_eq!(named.len(), PACKAGED_MODES.len());
    for file in named {
        assert!(PACKAGED_MODES.iter().any(|(name, _)| *name == file), "{file} is named in rk.md and not packaged");
    }
}

#[test]
fn every_mode_of_the_command_has_its_heading_in_a_packaged_file() {
    for heading in MODE_HEADINGS {
        let marker = format!("## {heading}\n");
        let packaged = std::iter::once(PACKAGED_COMMAND).chain(PACKAGED_MODES.iter().map(|(_, text)| *text));
        assert!(packaged.into_iter().any(|text| text.contains(&marker)), "no packaged file holds `{heading}`");
    }
}

#[test]
fn the_base_command_stays_under_its_word_limit() {
    let words = PACKAGED_COMMAND.split_whitespace().count();
    assert!(words < BASE_COMMAND_WORD_LIMIT, "rk.md has {words} words, the limit is {BASE_COMMAND_WORD_LIMIT}");
}
