use super::*;

fn some_id() -> Id {
    "1f2e3d4c-0000-4000-8000-000000000000".parse().unwrap()
}

#[test]
fn a_folder_is_the_slug_of_the_name_or_the_start_of_the_id() {
    assert_eq!(folder_segment("Release Plan (v2)", some_id()), "release-plan-v2");
    assert_eq!(folder_segment("***", some_id()), "1f2e3d4c");
    assert_eq!(folder_segment("..", some_id()), "1f2e3d4c");
}

#[test]
fn a_file_is_named_by_its_slug_its_content_hash_and_its_real_format() {
    let name = file_name(Path::new("/home/me/Login Flow.png"), b"abc", PictureFormat::Jpeg);

    assert_eq!(name, "login-flow-ba7816bf.jpg");
}

#[test]
fn a_file_whose_name_leaves_nothing_usable_is_called_image() {
    assert_eq!(file_name(Path::new("/home/me/§§§.png"), b"abc", PictureFormat::Png), "image-ba7816bf.png");
}

#[test]
fn different_content_under_one_name_gets_different_files() {
    let source = Path::new("/home/me/shot.png");

    assert_ne!(file_name(source, b"first", PictureFormat::Png), file_name(source, b"second", PictureFormat::Png));
}
