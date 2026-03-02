use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobStatus {
    Pending,
    Planning,
    Coding,
    Reviewing,
    Validating,
    Committing,
    Failed,
    Done,
}

impl JobStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            JobStatus::Pending => "PENDING",
            JobStatus::Planning => "PLANNING",
            JobStatus::Coding => "CODING",
            JobStatus::Reviewing => "REVIEWING",
            JobStatus::Validating => "VALIDATING",
            JobStatus::Committing => "COMMITTING",
            JobStatus::Failed => "FAILED",
            JobStatus::Done => "DONE",
        }
    }

    fn from_db(v: &str) -> Self {
        match v {
            "PLANNING" => JobStatus::Planning,
            "CODING" => JobStatus::Coding,
            "REVIEWING" => JobStatus::Reviewing,
            "VALIDATING" => JobStatus::Validating,
            "COMMITTING" => JobStatus::Committing,
            "FAILED" => JobStatus::Failed,
            "DONE" => JobStatus::Done,
            _ => JobStatus::Pending,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub issue_id: String,
    pub status: JobStatus,
    pub payload: String,
    pub retries: i32,
}

#[derive(Clone)]
pub struct SqliteDb {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteDb {
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn connection(&self) -> Arc<Mutex<Connection>> {
        self.conn.clone()
    }

    pub fn run_migrations(&self) -> Result<()> {
        self.conn.lock().expect("db lock").execute_batch(
            "
            CREATE TABLE IF NOT EXISTS jobs (
                id TEXT PRIMARY KEY,
                issue_id TEXT NOT NULL,
                status TEXT NOT NULL,
                payload TEXT NOT NULL,
                retries INTEGER NOT NULL DEFAULT 0,
                last_error TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS checkpoints (
                job_id TEXT NOT NULL,
                stage TEXT NOT NULL,
                data TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS execution_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                job_id TEXT NOT NULL,
                stage TEXT NOT NULL,
                input TEXT,
                output TEXT,
                model_used TEXT,
                duration_ms INTEGER,
                created_at TEXT NOT NULL
            );
            ",
        )?;
        Ok(())
    }
}

pub struct JobsRepository {
    conn: Arc<Mutex<Connection>>,
}

impl JobsRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn next_pending(&self) -> Option<Job> {
        let conn = self.conn.lock().expect("db lock");
        conn.query_row(
            "SELECT id, issue_id, status, payload, retries FROM jobs WHERE status='PENDING' ORDER BY created_at ASC LIMIT 1",
            [],
            |row| {
                Ok(Job {
                    id: row.get(0)?,
                    issue_id: row.get(1)?,
                    status: JobStatus::from_db(&row.get::<_, String>(2)?),
                    payload: row.get(3)?,
                    retries: row.get(4)?,
                })
            },
        ).ok()
    }

    pub fn update_status(&mut self, id: &str, status: JobStatus, error: Option<String>) {
        let _ = self.conn.lock().expect("db lock").execute(
            "UPDATE jobs SET status=?, last_error=?, updated_at=? WHERE id=?",
            params![status.as_str(), error, Utc::now().to_rfc3339(), id],
        );
    }

    pub fn increment_retries(&mut self, id: &str) {
        let _ = self.conn.lock().expect("db lock").execute(
            "UPDATE jobs SET retries=retries+1, updated_at=? WHERE id=?",
            params![Utc::now().to_rfc3339(), id],
        );
    }

    pub fn list_recent(&self) -> Vec<Job> {
        let conn = self.conn.lock().expect("db lock");
        let mut stmt = conn.prepare("SELECT id, issue_id, status, payload, retries FROM jobs ORDER BY created_at DESC LIMIT 20").expect("stmt");
        stmt.query_map([], |row| {
            Ok(Job {
                id: row.get(0)?,
                issue_id: row.get(1)?,
                status: JobStatus::from_db(&row.get::<_, String>(2)?),
                payload: row.get(3)?,
                retries: row.get(4)?,
            })
        })
        .expect("query")
        .flatten()
        .collect()
    }
}

pub struct CheckpointStore {
    conn: Arc<Mutex<Connection>>,
}

impl CheckpointStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
    pub fn get_latest(&self, job_id: &str, stage: &str) -> Option<String> {
        let conn = self.conn.lock().expect("db lock");
        conn.query_row(
            "SELECT data FROM checkpoints WHERE job_id=? AND stage=? ORDER BY created_at DESC LIMIT 1",
            params![job_id, stage],
            |row| row.get(0),
        )
        .ok()
    }

    pub fn save(&self, job_id: &str, stage: &str, data: &str) {
        let _ = self.conn.lock().expect("db lock").execute(
            "INSERT INTO checkpoints (job_id, stage, data, created_at) VALUES (?, ?, ?, ?)",
            params![job_id, stage, data, Utc::now().to_rfc3339()],
        );
    }
}

pub struct EvidenceStore {
    root: PathBuf,
}

impl EvidenceStore {
    pub fn new(root: String) -> Self {
        Self {
            root: PathBuf::from(root),
        }
    }

    pub fn write(&self, job_id: &str, stage: &str, content: &str) -> Result<()> {
        fs::create_dir_all(&self.root)?;
        let file = self.root.join(format!("{}-{}.txt", job_id, stage));
        fs::write(file, content)?;
        Ok(())
    }
}

pub struct ExecutionLogger {
    conn: Arc<Mutex<Connection>>,
}

impl ExecutionLogger {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn log_stage(
        &self,
        job_id: &str,
        stage: &str,
        input: &str,
        output: &str,
        model: &str,
        duration_ms: i64,
    ) {
        let _ = self.conn.lock().expect("db lock").execute(
            "INSERT INTO execution_log (job_id, stage, input, output, model_used, duration_ms, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![job_id, stage, input, output, model, duration_ms, Utc::now().to_rfc3339()],
        );
    }
}
