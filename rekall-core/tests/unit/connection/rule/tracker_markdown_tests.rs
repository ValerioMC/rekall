use super::*;
use crate::connection::Secret;

const LOGIN: &str = "acme/web/web-12/login-3f9a1c2b.png";

fn endpoint(kind: ConnectionKind, credential: &str) -> TrackerEndpoint {
    TrackerEndpoint { kind, site_url: "https://tracker.example.com".into(), proxy_url: None, location: "acme/web".into(), credential: Secret::new(credential) }
}

fn uploaded(markup: PictureMarkup, description: &str, remote: &str) -> String {
    let uploads = HashMap::from([(LOGIN.to_string(), remote.to_string())]);
    pictures_as_uploads(&format!("![{description}](/api/images/{LOGIN}#right)"), markup, &uploads)
}

#[test]
fn each_task_manager_has_its_own_markup() {
    let markup = |kind, credential| PictureMarkup::of(&endpoint(kind, credential)).unwrap();

    assert_eq!(markup(ConnectionKind::Github, "ghp"), PictureMarkup::MarkdownPicture);
    assert_eq!(markup(ConnectionKind::Gitlab, "glpat"), PictureMarkup::MarkdownPicture);
    assert_eq!(markup(ConnectionKind::Jira, "ada@example.com:token"), PictureMarkup::AttachmentLink);
    assert_eq!(markup(ConnectionKind::Jira, "pat:token"), PictureMarkup::AttachmentMacro);
    assert!(PictureMarkup::of(&endpoint(ConnectionKind::Grafana, "token")).is_err());
}

#[test]
fn an_uploaded_picture_shows_the_copy_without_its_alignment() {
    assert_eq!(uploaded(PictureMarkup::MarkdownPicture, "Login", "/uploads/1/login.png"), "![Login](/uploads/1/login.png)");
    assert_eq!(uploaded(PictureMarkup::AttachmentMacro, "Login", "login-3f9a1c2b.png"), "!login-3f9a1c2b.png!");
    assert_eq!(
        uploaded(PictureMarkup::AttachmentLink, "Login", "https://acme.atlassian.net/rest/api/3/attachment/content/7"),
        "[login-3f9a1c2b.png](https://acme.atlassian.net/rest/api/3/attachment/content/7)"
    );
}

#[test]
fn brackets_in_a_description_stay_inside_the_picture() {
    let uploads = HashMap::from([(LOGIN.to_string(), "https://copy".to_string())]);
    let markdown = format!(r"![a\](https://evil)](/api/images/{LOGIN})");

    let written = pictures_as_uploads(&markdown, PictureMarkup::MarkdownPicture, &uploads);

    assert_eq!(written, r"![a\](https://evil)](https://copy)");
}

#[test]
fn a_picture_not_uploaded_reads_as_its_description_or_its_file_name() {
    let markdown = format!("See ![Login Flow](/api/images/{LOGIN}) and ![](/api/images/{LOGIN}#center).");

    assert_eq!(pictures_as_descriptions(&markdown), "See Login Flow and login-3f9a1c2b.png.");
    assert_eq!(pictures_as_uploads(&markdown, PictureMarkup::MarkdownPicture, &HashMap::new()), "See Login Flow and login-3f9a1c2b.png.");
}
