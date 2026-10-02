use super::*;

fn version(text: &str) -> ReleaseVersion {
    text.parse().unwrap()
}

#[test]
fn a_tag_and_a_bare_version_name_the_same_release() {
    assert_eq!(version("v0.1.1"), version("0.1.1"));
}

#[test]
fn versions_are_ordered_by_number_not_by_text() {
    assert!(version("0.10.0") > version("0.9.9"));
    assert!(version("1.0.0") > version("0.99.99"));
    assert!(version("0.1.2") > version("0.1.1"));
}

#[test]
fn what_is_not_major_minor_patch_is_refused() {
    for text in ["latest", "1.0", "1.0.0.1", "1.0.0-rc1", "", "v"] {
        assert_eq!(text.parse::<ReleaseVersion>(), Err(InvalidVersion(text.to_string())), "{text}");
    }
}

#[test]
fn the_running_version_is_one_this_check_can_order() {
    assert!(RUNNING_VERSION.parse::<ReleaseVersion>().is_ok(), "{RUNNING_VERSION}");
}
