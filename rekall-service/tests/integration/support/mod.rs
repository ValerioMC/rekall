//! What the service tests share: a fresh database per test, rows to act on, and throwaway git
//! repositories driven by the real `git` binary.

#![allow(dead_code)]

use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use rekall_common::{Id, Instant};
use rekall_model::{company, project, task, task_step, ProjectStatus, TaskStepState};
use rekall_repository::Database;
use rekall_service::{Ctx, EventBus, Services};
use sea_orm::{ActiveModelTrait, IntoActiveModel};

pub struct World {
    pub database: Database,
    pub services: Services,
}

pub async fn world() -> World {
    let database = rekall_repository::open_temporary().await.unwrap();
    let services = Services::new(Ctx::new(database.conn.clone(), EventBus::new()));
    World { database, services }
}

pub async fn world_at(now: Instant) -> World {
    let database = rekall_repository::open_temporary().await.unwrap();
    let mut ctx = Ctx::new(database.conn.clone(), EventBus::new());
    ctx.clock = Arc::new(move || now);
    World { services: Services::new(ctx), database }
}

impl World {
    pub fn db(&self) -> &sea_orm::DatabaseConnection {
        &self.database.conn
    }

    pub async fn project(&self, label: &str, folder: Option<&str>, auto_commit: bool) -> project::Model {
        let now = Instant::now();
        let company = company::Model { id: Id::random(), name: format!("Company {label}"), description: None, created_at: now, updated_at: now };
        company.clone().into_active_model().insert(self.db()).await.unwrap();
        let project = project::Model {
            id: Id::random(),
            label: label.into(),
            title: label.into(),
            status: ProjectStatus::Active,
            icon: "folder".into(),
            description: None,
            blueprint_markdown: None,
            repo_folder: folder.map(str::to_string),
            auto_commit,
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
}

pub fn git(directory: &Path, args: &[&str]) -> String {
    let output = Command::new("git").arg("-C").arg(directory).args(args).output().unwrap();
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

pub fn init_with_identity(repo: &Path) {
    git(repo, &["init", "-q", "-b", "main"]);
    git(repo, &["config", "user.name", "Test"]);
    git(repo, &["config", "user.email", "test@example.com"]);
    git(repo, &["config", "commit.gpgsign", "false"]);
}

pub fn commit(repo: &Path, message: &str) {
    git(repo, &["-c", "user.name=Test", "-c", "user.email=test@example.com", "-c", "commit.gpgsign=false", "commit", "-q", "-m", message]);
}
