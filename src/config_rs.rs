use std::env;

use anyhow::{anyhow, Result};

#[derive(Clone)]
pub struct Config {
    pub db_path: String,
    pub evidence_path: String,
    pub repo_root: String,
    pub git_branch: String,
    pub anthropic_model: String,
    pub ollama_model: String,
    pub health_port: u16,
    pub poll_interval_ms: u64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            db_path: env::var("DB_PATH").unwrap_or_else(|_| "dual_agents.db".to_string()),
            evidence_path: env::var("EVIDENCE_PATH").unwrap_or_else(|_| "evidence".to_string()),
            repo_root: env::var("REPO_ROOT").unwrap_or_else(|_| ".".to_string()),
            git_branch: env::var("GIT_BRANCH").unwrap_or_else(|_| "main".to_string()),
            anthropic_model: env::var("ANTHROPIC_MODEL")
                .unwrap_or_else(|_| "claude-3-5-sonnet".to_string()),
            ollama_model: env::var("OLLAMA_MODEL").unwrap_or_else(|_| "codellama".to_string()),
            health_port: parse_env("HEALTH_PORT", 4001u16)?,
            poll_interval_ms: parse_env("POLL_INTERVAL_MS", 1000u64)?,
        })
    }
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
