use std::path::Path;

use super::*;

const PROJECT: &str = "/work/app";

#[test]
fn no_folder_runs_in_the_project_folder() {
    assert_eq!(working_folder(Path::new(PROJECT), None).unwrap(), Path::new(PROJECT));
}

#[test]
fn a_blank_folder_runs_in_the_project_folder() {
    assert_eq!(working_folder(Path::new(PROJECT), Some("  ")).unwrap(), Path::new(PROJECT));
}

#[test]
fn a_relative_folder_is_joined_to_the_project_folder() {
    assert_eq!(working_folder(Path::new(PROJECT), Some("rekall-ui")).unwrap(), Path::new("/work/app/rekall-ui"));
}

#[test]
fn a_folder_climbing_out_of_the_project_is_refused() {
    for folder in ["..", "../other", "ui/../../other"] {
        let failure = working_folder(Path::new(PROJECT), Some(folder)).unwrap_err();
        assert!(failure.is_illegal_argument(), "{folder}");
    }
}

#[test]
fn an_absolute_folder_is_refused() {
    let failure = working_folder(Path::new(PROJECT), Some("/etc")).unwrap_err();
    assert!(failure.is_illegal_argument());
}
