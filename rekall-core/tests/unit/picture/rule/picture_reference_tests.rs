use super::*;

const LOGIN: &str = "acme/web/web-12/login-3f9a1c2b.png";

fn files(markdown: &str) -> Vec<String> {
    PictureReference::all_in(markdown).into_iter().map(|reference| reference.picture.relative().to_string()).collect()
}

#[test]
fn a_picture_is_found_with_its_description_without_its_alignment() {
    let markdown = format!("Look:\n\n![Login *Flow*](/api/images/{LOGIN}#right)");

    let found = PictureReference::all_in(&markdown);

    assert_eq!(found.len(), 1);
    assert_eq!(found[0].picture.relative(), LOGIN);
    assert_eq!(found[0].description, "Login Flow");
    assert_eq!(&markdown[found[0].span.clone()], format!("![Login *Flow*](/api/images/{LOGIN}#right)"));
}

#[test]
fn pictures_in_lists_tables_links_and_references_are_found() {
    let markdown = format!(
        "- ![a](/api/images/{LOGIN})\n\n| x |\n|---|\n| ![b](/api/images/{LOGIN}#center) |\n\n[![c](/api/images/{LOGIN})](https://example.com)\n\n![d][ref]\n\n[ref]: /api/images/{LOGIN}\n"
    );

    assert_eq!(files(&markdown), [LOGIN; 4]);
}

#[test]
fn web_pictures_code_and_plain_links_are_no_pictures_of_the_folder() {
    let markdown = format!(
        "![web](https://example.com/a.png) `![code](/api/images/{LOGIN})` [link](/api/images/{LOGIN}) ![bad](/api/images/../x.png)\n\n```\n![fenced](/api/images/{LOGIN})\n```\n"
    );

    assert!(files(&markdown).is_empty());
}

#[test]
fn replacing_keeps_everything_around_the_pictures() {
    let markdown = format!("Über ![eins](/api/images/{LOGIN}) und ![zwei](/api/images/{LOGIN}#right) fertig.");

    let replaced = PictureReference::replaced_in(&markdown, |reference| reference.description.to_uppercase());

    assert_eq!(replaced, "Über EINS und ZWEI fertig.");
}
