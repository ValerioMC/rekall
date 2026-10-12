//! The files of Claude Code's home folder over HTTP: listed, opened, created, rewritten from the
//! version the editor holds, and deleted, with a memory's index line kept in step.

mod support;

use serde_json::json;
use std::path::Path;
use support::app;

fn claude_home(app: &support::App) -> std::path::PathBuf {
    app.folder.path().join("home/.claude")
}

fn put(root: &Path, relative: &str, body: &str) {
    let target = root.join(relative);
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
    std::fs::write(target, body).unwrap();
}

#[tokio::test]
async fn the_catalog_lists_what_is_on_disk() {
    let app = app().await;
    let root = claude_home(&app);
    put(&root, "CLAUDE.md", "rules");
    put(&root, "skills/docx/SKILL.md", "---\nname: docx\ndescription: word files\n---\n");
    put(&root, "commands/rk.md", "load a context");
    put(&root, "projects/-work-vega/memory/rule.md", "---\nname: Rule\n---\n");

    let catalog = app.get("/api/claude/resources").await;

    assert_eq!(catalog.status, 200);
    assert_eq!(catalog.get("instructions")["name"], "CLAUDE.md");
    assert_eq!(catalog.get("skills")[0]["description"], "word files");
    assert_eq!(catalog.get("commands")[0]["name"], "rk");
    assert_eq!(catalog.get("memoryGroups")[0]["id"], "-work-vega");
    assert_eq!(catalog.get("memoryGroups")[0]["memories"][0]["name"], "Rule");
    app.stop().await;
}

#[tokio::test]
async fn a_file_is_created_rewritten_and_deleted_with_its_index_line() {
    let app = app().await;
    let memory = "projects/-work-vega/memory/lean.md";

    let created = app.put("/api/claude/resources/file", json!({"path": memory, "body": "---\nname: Lean\ndescription: few rounds\n---\n"})).await;
    assert_eq!(created.status, 200, "{}", created.text);
    let index = app.get("/api/claude/resources/file?path=projects/-work-vega/memory/MEMORY.md").await;
    assert_eq!(index.str("body"), "- [Lean](lean.md) — few rounds\n");

    let opened = app.get(&format!("/api/claude/resources/file?path={memory}")).await;
    let version = opened.get("summary")["modifiedAt"].clone();
    let rewritten = app.put("/api/claude/resources/file", json!({"path": memory, "body": "new", "expectedModifiedAt": version})).await;
    assert_eq!(rewritten.status, 200, "{}", rewritten.text);
    assert_eq!(rewritten.str("body"), "new");

    let removed = app.delete(&format!("/api/claude/resources/file?path={memory}")).await;
    assert_eq!(removed.status, 204);
    assert_eq!(app.get(&format!("/api/claude/resources/file?path={memory}")).await.status, 404);
    assert_eq!(app.get("/api/claude/resources/file?path=projects/-work-vega/memory/MEMORY.md").await.str("body"), "");
    app.stop().await;
}

#[tokio::test]
async fn a_stale_save_is_a_conflict_and_leaves_the_file_alone() {
    let app = app().await;
    let first = app.put("/api/claude/resources/file", json!({"path": "CLAUDE.md", "body": "one"})).await;
    let opened_at = first.get("summary")["modifiedAt"].clone();
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    std::fs::write(claude_home(&app).join("CLAUDE.md"), "written by a session").unwrap();

    let stale = app.put("/api/claude/resources/file", json!({"path": "CLAUDE.md", "body": "mine", "expectedModifiedAt": opened_at})).await;

    assert_eq!(stale.status, 409);
    assert_eq!(std::fs::read_to_string(claude_home(&app).join("CLAUDE.md")).unwrap(), "written by a session");
    app.stop().await;
}

#[tokio::test]
async fn only_the_known_places_are_reachable() {
    let app = app().await;
    assert_eq!(app.get("/api/claude/resources/file?path=../escape.md").await.status, 400);
    assert_eq!(app.get("/api/claude/resources/file?path=settings.json").await.status, 400);
    assert_eq!(app.get("/api/claude/resources/file").await.status, 400);
    assert_eq!(app.put("/api/claude/resources/file", json!({"path": "../escape.md", "body": "x"})).await.status, 400);
    assert_eq!(app.put("/api/claude/resources/file", json!({"body": "x"})).await.status, 400);
    assert_eq!(app.delete("/api/claude/resources/file?path=commands/none.md").await.status, 404);
    app.stop().await;
}

#[tokio::test]
async fn the_global_instructions_cannot_be_deleted() {
    let app = app().await;
    put(&claude_home(&app), "CLAUDE.md", "rules");
    assert_eq!(app.delete("/api/claude/resources/file?path=CLAUDE.md").await.status, 409);
    assert!(claude_home(&app).join("CLAUDE.md").exists());
    app.stop().await;
}
