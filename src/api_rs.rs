use std::sync::{Arc, Mutex};

use anyhow::Result;
use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};
use serde_json::json;

use crate::{config_rs::Config, persistence_rs::JobsRepository};

#[derive(Clone)]
struct AppState {
    jobs: Arc<Mutex<JobsRepository>>,
}

pub async fn serve(config: Config, jobs: Arc<Mutex<JobsRepository>>) -> Result<()> {
    let app_state = AppState { jobs };
    let app = Router::new()
        .route("/health", get(health))
        .route("/jobs", get(list_jobs))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", config.health_port)).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> impl IntoResponse {
    Json(json!({"status": "ok", "engine": "rust"}))
}

async fn list_jobs(State(state): State<AppState>) -> impl IntoResponse {
    let jobs = state.jobs.lock().expect("jobs lock").list_recent();
    Json(json!(jobs))
}
