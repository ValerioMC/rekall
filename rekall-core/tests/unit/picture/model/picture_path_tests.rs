use super::*;

#[test]
fn a_task_picture_and_a_page_picture_are_paths_with_their_address() {
    let task = PicturePath::parse("acme/web/web-12/login-flow-3f9a1c2b.png").unwrap();
    let page = PicturePath::parse("acme/web/pages/release.plan/a_b-0011aabb.svg").unwrap();

    assert_eq!(task.address(), "/api/images/acme/web/web-12/login-flow-3f9a1c2b.png");
    assert_eq!(task.format(), PictureFormat::Png);
    assert_eq!(page.relative(), "acme/web/pages/release.plan/a_b-0011aabb.svg");
    assert_eq!(page.format(), PictureFormat::Svg);
}

#[test]
fn every_kept_format_is_read_from_its_extension() {
    let formats: Vec<PictureFormat> = ["png", "jpg", "gif", "webp", "svg"]
        .iter()
        .map(|extension| PicturePath::parse(&format!("acme/web/web-12/a.{extension}")).unwrap().format())
        .collect();

    assert_eq!(formats, [PictureFormat::Png, PictureFormat::Jpeg, PictureFormat::Gif, PictureFormat::Webp, PictureFormat::Svg]);
}

#[test]
fn a_path_that_could_leave_the_folder_is_refused() {
    for path in [
        "acme/web/../a.png",
        "acme/web/./a.png",
        "../web/web-12/a.png",
        "/acme/web/web-12/a.png",
        "acme//web-12/a.png",
        "acme/web/web-12/..\\a.png",
        "acme/web/web-12/",
    ] {
        assert!(PicturePath::parse(path).is_err(), "{path}");
    }
}

#[test]
fn a_path_of_the_wrong_depth_extension_or_alphabet_is_refused() {
    for path in [
        "acme/web/a.png",
        "acme/web/pages/guide/deeper/a.png",
        "acme/web/web-12/a.bmp",
        "acme/web/web-12/a.jpeg",
        "acme/web/web-12/a.PNG",
        "acme/web/web-12/png",
        "acme/web/web-12/Login.png",
        "acme/web/web-12/login flow.png",
    ] {
        assert!(PicturePath::parse(path).is_err(), "{path}");
    }
}
