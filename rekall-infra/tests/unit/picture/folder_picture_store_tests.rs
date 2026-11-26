use tempfile::TempDir;

use super::*;

const LIMIT: u64 = 1024 * 1024;

fn path() -> PicturePath {
    PicturePath::parse("acme/web/web-12/login-3f9a1c2b.png").unwrap()
}

fn store_in(folder: &TempDir) -> FolderPictureStore {
    let location = DatabaseLocation::of(&folder.path().join("rekall.db")).unwrap();
    FolderPictureStore::new(Some(&location))
}

fn a_file(folder: &TempDir, name: &str, bytes: &[u8]) -> PathBuf {
    let file = folder.path().join(name);
    std::fs::write(&file, bytes).unwrap();
    file
}

#[tokio::test]
async fn a_picture_is_written_under_the_images_folder_beside_the_database_and_read_back() {
    let folder = tempfile::tempdir().unwrap();
    let store = store_in(&folder);

    store.write(&path(), b"pixels").await.unwrap();

    assert_eq!(std::fs::read(folder.path().join("images/acme/web/web-12/login-3f9a1c2b.png")).unwrap(), b"pixels");
    assert_eq!(store.read(&path()).await.unwrap(), b"pixels");
}

#[tokio::test]
async fn writing_a_picture_again_renews_its_modification_time() {
    let folder = tempfile::tempdir().unwrap();
    let store = store_in(&folder);
    let file = folder.path().join("images").join(path().relative());
    store.write(&path(), b"pixels").await.unwrap();
    let long_ago = std::time::SystemTime::UNIX_EPOCH;
    std::fs::File::options().write(true).open(&file).unwrap().set_modified(long_ago).unwrap();

    store.write(&path(), b"pixels").await.unwrap();

    assert!(std::fs::metadata(&file).unwrap().modified().unwrap() > long_ago);
}

#[tokio::test]
async fn a_picture_that_is_not_there_is_not_found() {
    let folder = tempfile::tempdir().unwrap();
    let store = store_in(&folder);

    assert!(matches!(store.read(&path()).await, Err(RekallError::NotFound(_))));
}

#[tokio::test]
async fn a_chosen_file_is_read_up_to_the_limit() {
    let folder = tempfile::tempdir().unwrap();
    let source = a_file(&folder, "shot.png", &[7; 4]);

    assert_eq!(store_in(&folder).read_source(&source, 4).await.unwrap(), [7; 4]);
}

#[tokio::test]
async fn a_relative_missing_oversized_or_folder_source_is_the_callers_mistake() {
    let folder = tempfile::tempdir().unwrap();
    let store = store_in(&folder);
    let large = a_file(&folder, "large.png", &vec![0; 2 * LIMIT as usize]);
    let sources = [PathBuf::from("shot.png"), folder.path().join("missing.png"), folder.path().to_path_buf(), large];

    for source in sources {
        let refusal = store.read_source(&source, LIMIT).await.unwrap_err();
        assert!(matches!(refusal, RekallError::IllegalArgument(_)), "{}: {refusal}", source.display());
    }
}

#[tokio::test]
async fn without_a_database_file_every_call_is_a_conflict() {
    let folder = tempfile::tempdir().unwrap();
    let source = a_file(&folder, "shot.png", b"pixels");
    let store = FolderPictureStore::new(None);

    assert!(matches!(store.read_source(&source, LIMIT).await, Err(RekallError::Conflict(_))));
    assert!(matches!(store.write(&path(), b"pixels").await, Err(RekallError::Conflict(_))));
    assert!(matches!(store.read(&path()).await, Err(RekallError::Conflict(_))));
}

#[test]
fn the_folder_is_the_real_images_folder_before_and_after_it_exists() {
    let folder = tempfile::tempdir().unwrap();
    let database_folder = folder.path().join("db");
    std::fs::create_dir(&database_folder).unwrap();
    let link = folder.path().join("linked-db");
    std::os::unix::fs::symlink(&database_folder, &link).unwrap();
    let store = FolderPictureStore::new(Some(&DatabaseLocation::of(&link.join("rekall.db")).unwrap()));
    let real = database_folder.canonicalize().unwrap().join("images");

    assert_eq!(store.folder(), Some(real.clone()));
    std::fs::create_dir(database_folder.join("images")).unwrap();
    assert_eq!(store.folder(), Some(real));
}

#[test]
fn without_a_database_file_there_is_no_folder() {
    assert_eq!(FolderPictureStore::new(None).folder(), None);
}

#[tokio::test]
async fn pictures_are_listed_with_their_size_and_when_they_were_written() {
    let folder = tempfile::tempdir().unwrap();
    let store = store_in(&folder);
    store.write(&path(), b"pixels").await.unwrap();
    std::fs::write(folder.path().join("images/acme/web/web-12/notes.txt"), "not a picture").unwrap();
    let file = folder.path().join("images").join(path().relative());
    let long_ago = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(86_400);
    std::fs::File::options().write(true).open(&file).unwrap().set_modified(long_ago).unwrap();

    let listed = store.list().await.unwrap();

    assert_eq!(listed, [StoredPicture { path: path(), size_bytes: 6, modified_at: Instant::from_epoch_seconds(86_400).unwrap() }]);
}

#[tokio::test]
async fn a_store_with_no_picture_yet_lists_nothing() {
    let folder = tempfile::tempdir().unwrap();

    assert!(store_in(&folder).list().await.unwrap().is_empty());
}

#[tokio::test]
async fn deleting_a_picture_removes_the_folders_it_leaves_empty_but_never_the_images_folder() {
    let folder = tempfile::tempdir().unwrap();
    let store = store_in(&folder);
    store.write(&path(), b"pixels").await.unwrap();

    store.delete(&path()).await.unwrap();

    assert!(!folder.path().join("images/acme").exists());
    assert!(folder.path().join("images").is_dir());
}

#[tokio::test]
async fn deleting_a_picture_keeps_a_folder_that_holds_something_else() {
    let folder = tempfile::tempdir().unwrap();
    let store = store_in(&folder);
    let sibling = PicturePath::parse("acme/web/blueprint/map-3f9a1c2b.png").unwrap();
    store.write(&path(), b"pixels").await.unwrap();
    store.write(&sibling, b"pixels").await.unwrap();

    store.delete(&path()).await.unwrap();

    assert!(!folder.path().join("images/acme/web/web-12").exists());
    assert_eq!(store.read(&sibling).await.unwrap(), b"pixels");
}

#[tokio::test]
async fn deleting_a_picture_that_is_already_gone_succeeds() {
    let folder = tempfile::tempdir().unwrap();

    assert!(store_in(&folder).delete(&path()).await.is_ok());
}

#[tokio::test]
async fn a_database_that_is_not_a_file_lists_and_deletes_nothing() {
    let store = FolderPictureStore::new(None);

    assert!(matches!(store.list().await, Err(RekallError::Conflict(_))));
    assert!(matches!(store.delete(&path()).await, Err(RekallError::Conflict(_))));
}
