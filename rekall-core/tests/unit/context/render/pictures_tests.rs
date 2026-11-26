use std::path::PathBuf;

use super::*;

const SHOWN: &str = "Build it like this ![Login Flow](/api/images/acme/web/web-12/login-flow-3f9a1c2b.png#right)\n";

fn pictures() -> SessionPictures {
    SessionPictures::new(Some(PathBuf::from("/data/rekall/images")))
}

#[test]
fn a_text_showing_pictures_ends_once_with_their_folder_and_keeps_its_addresses() {
    let text = format!("{SHOWN}\n{SHOWN}");

    let handed = pictures().appended_to(text.clone());

    assert_eq!(handed.matches("<pictures folder=").count(), 1);
    assert_eq!(
        handed,
        format!(
            "{}\n\n<pictures folder=\"/data/rekall/images\">A markdown picture `/api/images/<path>` is the file `<folder>/<path>`. \
             Open it with Read when the work needs it. In text you save, keep the address, never the file path.</pictures>\n",
            text.trim_end()
        )
    );
}

#[test]
fn a_text_without_a_picture_is_handed_as_it_is() {
    let text = "![Logo](https://example.com/logo.png) and docs/sketch.png\n".to_string();

    assert_eq!(pictures().appended_to(text.clone()), text);
}

#[test]
fn without_a_folder_no_block_is_added() {
    assert_eq!(SessionPictures::default().appended_to(SHOWN.to_string()), SHOWN);
}
