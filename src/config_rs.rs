use std::env;

use anyhow::{anyhow, Result};

#[derive(Clone)]
pub struct Config {
    pub db_path: String,
    pub evidence_path: String,
    pub repo_root: String,
    pub git_branch: String,
    pub git_remote: String,
    pub anthropic_model: String,
    pub anthropic_api_key: Option<String>,
    pub ollama_model: String,
    pub ollama_base_url: String,
    pub linear_api_key: String,
    pub linear_team_id: String,
    pub linear_done_state_type: String,
    pub health_port: u16,
    pub poll_interval_ms: u64,
    pub max_review_iterations: u8,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let _ = dotenvy::dotenv();
        Ok(Self {
            db_path: env::var("DB_PATH").unwrap_or_else(|_| "dual_agents.db".to_string()),
            evidence_path: env::var("EVIDENCE_PATH").unwrap_or_else(|_| "evidence".to_string()),
            repo_root: env::var("REPO_ROOT").unwrap_or_else(|_| ".".to_string()),
            git_branch: env::var("GIT_BRANCH").unwrap_or_else(|_| "main".to_string()),
            git_remote: env::var("GIT_REMOTE").unwrap_or_else(|_| "origin".to_string()),
            anthropic_model: env::var("ANTHROPIC_MODEL")
                .unwrap_or_else(|_| "claude-3-5-sonnet-20241022".to_string()),
            anthropic_api_key: env::var("ANTHROPIC_API_KEY").ok(),
            ollama_model: env::var("OLLAMA_MODEL").unwrap_or_else(|_| "codellama".to_string()),
            ollama_base_url: env::var("OLLAMA_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:11434".to_string()),
            linear_api_key: required("LINEAR_API_KEY")?,
            linear_team_id: required("LINEAR_TEAM_ID")?,
            linear_done_state_type: env::var("LINEAR_DONE_STATE_TYPE")
                .unwrap_or_else(|_| "completed".to_string()),
            health_port: parse_env("HEALTH_PORT", 4001u16)?,
            poll_interval_ms: parse_env("POLL_INTERVAL_MS", 1000u64)?,
            max_review_iterations: parse_env("MAX_REVIEW_ITERATIONS", 2u8)?,
        })
    }
}

fn required(key: &str) -> Result<String> {
    env::var(key).map_err(|_| anyhow!("variável obrigatória ausente: {key}"))
}

fn parse_env<T>(key: &str, default: T) -> Result<T>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    match env::var(key) {
        Ok(raw) => raw
            .parse::<T>()
            .map_err(|err| anyhow!("invalid value for {key}: {raw} ({err})")),
        Err(_) => Ok(default),
    }
}
