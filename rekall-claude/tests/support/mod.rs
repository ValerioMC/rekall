//! What the terminal and run-queue tests share: a fresh database, a project whose folder exists,
//! and a stand-in `claude` that echoes its arguments and input instead of starting a TUI.

#![allow(dead_code)]

use std::collections::HashMap;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rekall_claude::cli::{ClaudeCli, EnvironmentSource};
use rekall_claude::pty::{Listener, PtyTerminalManager};
use rekall_claude::ClaudeConfig;
use rekall_common::{Id, Instant};
use rekall_model::{company, project, task, task_step, ProjectStatus, TaskStepState};
use rekall_repository::Database;
use rekall_service::{Ctx, EventBus, Services};
use sea_orm::{ActiveModelTrait, IntoActiveModel};
use tempfile::TempDir;

/// Prints its arguments, then echoes each line it reads; `exit` ends it with code 3.
pub const STUB_CLAUDE: &str = r#"#!/bin/sh
echo "ARGS:$*"
while IFS= read -r line; do
  if [ "$line" = "exit" ]; then
    echo "BYE"
    exit 3
  fi
  echo "GOT:$line"
done
"#;

pub struct FixedEnvironment;

#[async_trait::async_trait]
impl EnvironmentSource for FixedEnvironment {
    async fn current(&self) -> HashMap<String, String> {
        HashMap::from([("PATH".to_string(), "/usr/bin:/bin".to_string())])
    }
}

pub struct World {
    pub database: Database,
    pub services: Services,
    pub events: EventBus,
    pub folder: TempDir,
    pub home: TempDir,
    pub claude: PathBuf,
}

pub async fn world() -> World {
    let database = rekall_repository::open_temporary().await.unwrap();
    let events = EventBus::new();
    let services = Services::new(Ctx::new(database.conn.clone(), events.clone()));
    let folder = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let claude = home.path().join("stub-claude");
    std::fs::write(&claude, STUB_CLAUDE).unwrap();
    std::fs::set_permissions(&claude, std::fs::Permissions::from_mode(0o755)).unwrap();
    World { database, services, events, folder, home, claude }
}

impl World {
    pub fn db(&self) -> &sea_orm::DatabaseConnection {
        &self.database.conn
    }

    pub fn terminals(&self, max_sessions: usize) -> Arc<PtyTerminalManager> {
        let config = ClaudeConfig { max_sessions, home: self.home.path().to_path_buf(), ..ClaudeConfig::default() };
        let cli = ClaudeCli::new(self.claude.to_str(), self.home.path().to_path_buf(), Arc::new(FixedEnvironment));
        PtyTerminalManager::new(
            self.services.terminal_launch.clone(),
            cli,
            self.services.steps.clone(),
            self.services.review.clone(),
            &config,
        )
    }

    pub async fn project(&self, label: &str, folder: Option<&Path>) -> project::Model {
        let now = Instant::now();
        let company = company::Model { id: Id::random(), name: format!("Company {label}"), description: None, created_at: now, updated_at: now };
        company.clone().into_active_model().insert(self.db()).await.unwrap();
        let project = project::Model {
            id: Id::random(),
            label: label.into(),
            title: label.into(),
            status: ProjectStatus::Active,
            description: None,
            blueprint_markdown: None,
            repo_folder: folder.map(|f| f.to_string_lossy().into_owned()),
            auto_commit: false,
            company_id: company.id,
            created_at: now,
            updated_at: now,
        };
        project.clone().into_active_model().insert(self.db()).await.unwrap();
        project
    }

    pub async fn task(&self, project: &project::Model, label: &str) -> task::Model {
        let task = task::Model::new(label.into(), format!("Title of {label}"), project.id, Instant::now());
        task.clone().into_active_model().insert(self.db()).await.unwrap();
        task
    }

    pub async fn step(&self, task: &task::Model, title: &str, position: i32, state: TaskStepState) -> task_step::Model {
        let mut step = task_step::Model::new(task.id, title.into(), position, Instant::now());
        step.state = state;
        step.clone().into_active_model().insert(self.db()).await.unwrap();
        step
    }

    pub async fn step_state(&self, task: &task::Model, step: &task_step::Model) -> TaskStepState {
        let steps = self.services.steps.find_by_task(task.id).await.unwrap();
        steps.into_iter().find(|s| s.id == step.id).unwrap().state
    }
}

/// Collects what a terminal prints and how it ended.
#[derive(Default)]
pub struct Recorder {
    pub output: Mutex<Vec<u8>>,
    pub ended: Mutex<Option<(i32, String)>>,
}

impl Recorder {
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.output.lock().unwrap()).into_owned()
    }
}

impl Listener for Recorder {
    fn output(&self, data: &[u8]) {
        self.output.lock().unwrap().extend_from_slice(data);
    }

    fn ended(&self, exit_code: i32, detail: &str) {
        *self.ended.lock().unwrap() = Some((exit_code, detail.to_string()));
    }
}

/// Poll `condition` every 20ms for up to five seconds.
pub async fn eventually<F, Fut>(what: &str, mut condition: F)
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    while tokio::time::Instant::now() < deadline {
        if condition().await {
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("timed out waiting for {what}");
}
