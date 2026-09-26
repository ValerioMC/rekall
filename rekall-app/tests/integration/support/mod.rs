//! The whole application on a real port, the way `@SpringBootTest(webEnvironment = RANDOM_PORT)`
//! ran it: a file database and a `rekall.home` of the test's own, a console bundle that is one
//! `index.html`, and an HTTP client that reports every status instead of failing on one.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use rekall_app::{AppConfig, Properties, Running, StartOptions};
use rekall_service::Services;
use sea_orm::{ConnectionTrait, DbBackend, Statement, Value as DbValue};
use serde_json::{json, Value};
use tempfile::TempDir;

pub struct App {
    pub running: Option<Running>,
    pub base: String,
    pub client: reqwest::Client,
    pub folder: TempDir,
}

pub struct Resp {
    pub status: u16,
    pub body: Value,
    pub text: String,
    pub headers: reqwest::header::HeaderMap,
}

impl Resp {
    pub fn get(&self, key: &str) -> &Value {
        &self.body[key]
    }

    pub fn str(&self, key: &str) -> String {
        match &self.body[key] {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        }
    }

    pub fn detail(&self) -> String {
        self.str("detail")
    }

    pub fn list(&self) -> Vec<Value> {
        self.body.as_array().cloned().unwrap_or_default()
    }
}

pub async fn app() -> App {
    app_with(&[], StartOptions { no_restart: true, ..StartOptions::default() }).await
}

pub async fn app_with(properties: &[(&str, &str)], options: StartOptions) -> App {
    let folder = tempfile::tempdir().unwrap();
    let ui = folder.path().join("ui");
    std::fs::create_dir_all(&ui).unwrap();
    std::fs::write(ui.join("index.html"), "<!doctype html><html><body><div id=\"app\"></div></body></html>").unwrap();
    let home = folder.path().join("home");
    let db_url = format!("sqlite:{}", folder.path().join("db/rekall.db").display());
    let mut pairs: Vec<(String, String)> = vec![
        ("server.port".into(), "0".into()),
        ("rekall.home".into(), home.join(".rekall").to_string_lossy().into_owned()),
        ("user.home".into(), home.to_string_lossy().into_owned()),
        ("spring.datasource.url".into(), db_url),
        ("rekall.backup.enabled".into(), "false".into()),
        ("rekall.terminal.sweep-minutes".into(), "60".into()),
        ("rekall.ui.dist".into(), ui.to_string_lossy().into_owned()),
    ];
    for (name, value) in properties {
        pairs.retain(|(n, _)| n != name);
        pairs.push((name.to_string(), value.to_string()));
    }
    let borrowed: Vec<(&str, &str)> = pairs.iter().map(|(n, v)| (n.as_str(), v.as_str())).collect();
    let config = AppConfig::from_sources(&Properties::of(&borrowed));
    let running = rekall_app::start(config, options).await.expect("the application starts");
    let base = format!("http://localhost:{}", running.port);
    let client = reqwest::Client::builder().no_proxy().build().unwrap();
    App { running: Some(running), base, client, folder }
}

impl App {
    pub fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base)
    }

    pub fn services(&self) -> Services {
        self.running.as_ref().unwrap().services().expect("an instance is up")
    }

    pub async fn stop(mut self) {
        if let Some(running) = self.running.take() {
            running.stop().await;
        }
    }

    pub async fn send(&self, method: reqwest::Method, path: &str, body: Option<Value>) -> Resp {
        let mut request = self.client.request(method, self.url(path));
        if let Some(body) = body {
            request = request.json(&body);
        }
        into_resp(request.send().await.expect("the server answers")).await
    }

    pub async fn get(&self, path: &str) -> Resp {
        self.send(reqwest::Method::GET, path, None).await
    }

    pub async fn post(&self, path: &str, body: Value) -> Resp {
        self.send(reqwest::Method::POST, path, Some(body)).await
    }

    pub async fn post_empty(&self, path: &str) -> Resp {
        self.send(reqwest::Method::POST, path, None).await
    }

    pub async fn put(&self, path: &str, body: Value) -> Resp {
        self.send(reqwest::Method::PUT, path, Some(body)).await
    }

    pub async fn patch(&self, path: &str, body: Value) -> Resp {
        self.send(reqwest::Method::PATCH, path, Some(body)).await
    }

    pub async fn delete(&self, path: &str) -> Resp {
        self.send(reqwest::Method::DELETE, path, None).await
    }

    // ------------------------------------------------------------ the database, as jdbc read it

    pub async fn query(&self, sql: &str, params: Vec<DbValue>) -> Vec<sea_orm::QueryResult> {
        let db = self.services().ctx.db.clone();
        db.query_all_raw(Statement::from_sql_and_values(DbBackend::Sqlite, sql, params)).await.unwrap()
    }

    pub async fn count(&self, sql: &str, params: Vec<DbValue>) -> i64 {
        let rows = self.query(sql, params).await;
        rows[0].try_get_by_index::<i64>(0).unwrap()
    }

    pub async fn string(&self, sql: &str, params: Vec<DbValue>) -> Option<String> {
        let rows = self.query(sql, params).await;
        rows.first().and_then(|row| row.try_get_by_index::<Option<String>>(0).ok().flatten())
    }

    pub async fn execute(&self, sql: &str, params: Vec<DbValue>) {
        let db = self.services().ctx.db.clone();
        db.execute_raw(Statement::from_sql_and_values(DbBackend::Sqlite, sql, params)).await.unwrap();
    }

    // ------------------------------------------------------------ RekallEndToEndTest's helpers

    pub async fn a_company(&self, name: &str) -> String {
        id(&self.post("/api/companies", json!({ "name": name })).await)
    }

    pub async fn a_project(&self, company_id: &str, label: &str, status: &str) -> String {
        id(&self
            .post("/api/projects", json!({ "label": label, "title": label, "status": status, "companyId": company_id }))
            .await)
    }

    pub async fn a_project_in(&self, company_id: &str, label: &str, folder: &Path) -> String {
        id(&self
            .post(
                "/api/projects",
                json!({ "label": label, "title": "Vega", "status": "ACTIVE", "repoFolder": folder.to_string_lossy(), "companyId": company_id }),
            )
            .await)
    }

    pub async fn a_task(&self, project_id: &str, label: &str) -> String {
        id(&self
            .post("/api/tasks", json!({ "label": label, "title": label, "status": "TODO", "projectId": project_id }))
            .await)
    }

    /// Creates a step and promotes it out of draft.
    pub async fn a_step(&self, task_id: &str, title: &str, body: Option<&str>) -> String {
        let step_id = self.a_draft_step(task_id, title, body).await;
        self.patch(&format!("/api/steps/{step_id}"), json!({ "draft": false })).await;
        step_id
    }

    pub async fn a_draft_step(&self, task_id: &str, title: &str, body: Option<&str>) -> String {
        id(&self.post(&format!("/api/tasks/{task_id}/steps"), json!({ "title": title, "bodyMarkdown": body })).await)
    }

    pub async fn get_task(&self, task_id: &str) -> Value {
        self.get(&format!("/api/tasks/{task_id}")).await.body
    }

    pub async fn review(&self, task_id: &str, body: Value) -> Resp {
        self.patch(&format!("/api/tasks/{task_id}/review"), body).await
    }

    pub async fn labels_of_steps(&self, task_id: &str) -> Vec<String> {
        self.get(&format!("/api/tasks/{task_id}/steps"))
            .await
            .list()
            .iter()
            .map(|step| step["title"].as_str().unwrap_or("null").to_string())
            .collect()
    }

    pub async fn documents_on(&self, task_id: &str) -> Vec<Value> {
        self.get(&format!("/api/documents?taskId={task_id}")).await.list()
    }

    pub async fn rpc(&self, method: &str, params: Value) -> Value {
        self.post("/mcp", json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params })).await.body
    }

    pub async fn call_tool(&self, name: &str, arguments: Value) -> String {
        let answer = self.rpc("tools/call", json!({ "name": name, "arguments": arguments })).await;
        answer["result"]["content"][0]["text"].as_str().unwrap_or_default().to_string()
    }

    /// A stateless-era request: no handshake, every field mirrored into a header.
    pub async fn modern_rpc(&self, method: &str, name: Option<&str>, params: Value) -> Resp {
        let mut request = self
            .client
            .post(self.url("/mcp"))
            .header("MCP-Protocol-Version", "2026-07-28")
            .header("Mcp-Method", method);
        if let Some(name) = name {
            request = request.header("Mcp-Name", name);
        }
        let request = request.json(&json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params }));
        into_resp(request.send().await.unwrap()).await
    }
}

async fn into_resp(response: reqwest::Response) -> Resp {
    let status = response.status().as_u16();
    let headers = response.headers().clone();
    let text = response.text().await.unwrap_or_default();
    let body = serde_json::from_str(&text).unwrap_or(Value::Null);
    Resp { status, body, text, headers }
}

/// `id(ResponseEntity)`: the create answered 201, and this is its id.
pub fn id(response: &Resp) -> String {
    assert_eq!(response.status, 201, "expected a create, got {}: {}", response.status, response.text);
    response.str("id")
}

pub fn uuid_value(id: &str) -> DbValue {
    DbValue::String(Some(id.to_string()))
}

// ---------------------------------------------------------------- git

pub fn run_git(directory: &Path, args: &[&str]) -> String {
    let output = Command::new("git").arg("-C").arg(directory).args(args).output().unwrap();
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn committed(repo: &Path, subject: &str) {
    std::fs::write(repo.join("README.md"), subject).unwrap();
    run_git(repo, &["add", "README.md"]);
    run_git(
        repo,
        &["-c", "user.name=Test", "-c", "user.email=test@example.com", "-c", "commit.gpgsign=false", "commit", "-q", "-m", subject],
    );
}

pub fn a_git_repo_with_commits(subjects: &[&str]) -> TempDir {
    let repo = tempfile::tempdir().unwrap();
    run_git(repo.path(), &["init", "-q"]);
    run_git(repo.path(), &["config", "commit.gpgsign", "false"]);
    for subject in subjects {
        committed(repo.path(), subject);
    }
    repo
}

pub fn a_git_repo_with_one_commit(subject: &str) -> TempDir {
    let repo = tempfile::tempdir().unwrap();
    run_git(repo.path(), &["init", "-q"]);
    // The auto-commit path commits as whoever git is configured to be in the folder.
    run_git(repo.path(), &["config", "user.name", "Test"]);
    run_git(repo.path(), &["config", "user.email", "test@example.com"]);
    run_git(repo.path(), &["config", "commit.gpgsign", "false"]);
    committed(repo.path(), subject);
    repo
}

pub fn path_of(dir: &TempDir) -> PathBuf {
    dir.path().to_path_buf()
}

/// Poll every 100ms until `condition` holds, for up to `seconds`.
pub async fn await_until<F, Fut>(seconds: u64, mut condition: F)
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let deadline = tokio::time::Instant::now() + Duration::from_secs(seconds);
    while tokio::time::Instant::now() < deadline {
        if condition().await {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("condition not met within {seconds}s");
}

/// `assertThat(text).contains(...)` with the text in the failure.
#[macro_export]
macro_rules! assert_contains {
    ($text:expr, $($needle:expr),+ $(,)?) => {{
        let text: &str = &$text;
        $( assert!(text.contains($needle), "expected {:?} in:\n{}", $needle, text); )+
    }};
}

#[macro_export]
macro_rules! assert_lacks {
    ($text:expr, $($needle:expr),+ $(,)?) => {{
        let text: &str = &$text;
        $( assert!(!text.contains($needle), "did not expect {:?} in:\n{}", $needle, text); )+
    }};
}

// ---------------------------------------------------------------- the event stream

/// `/api/steps/stream` read line by line, as `HttpResponse.BodyHandlers.ofLines()` did.
pub struct EventLines {
    lines: tokio::sync::mpsc::UnboundedReceiver<String>,
    reader: tokio::task::JoinHandle<()>,
}

impl App {
    pub async fn open_stream(&self) -> EventLines {
        let (sender, lines) = tokio::sync::mpsc::unbounded_channel();
        let mut response = self.client.get(self.url("/api/steps/stream")).send().await.unwrap();
        let reader = tokio::spawn(async move {
            let mut pending = String::new();
            while let Ok(Some(chunk)) = response.chunk().await {
                pending.push_str(&String::from_utf8_lossy(&chunk));
                while let Some(end) = pending.find('\n') {
                    let line: String = pending.drain(..=end).collect();
                    let _ = sender.send(line.trim_end_matches(['\n', '\r']).to_string());
                }
            }
        });
        EventLines { lines, reader }
    }
}

impl EventLines {
    /// The next line holding `needle`, skipping the others, within `seconds`.
    pub async fn await_line(&mut self, needle: &str, seconds: u64) -> String {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(seconds);
        loop {
            match tokio::time::timeout_at(deadline, self.lines.recv()).await {
                Ok(Some(line)) if line.contains(needle) => return line,
                Ok(Some(_)) => continue,
                _ => panic!("No event-stream line containing '{needle}' within {seconds}s"),
            }
        }
    }
}

impl Drop for EventLines {
    fn drop(&mut self) {
        self.reader.abort();
    }
}
