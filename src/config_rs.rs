use std::env;

#[derive(Clone)]
pub struct Config {
    pub db_path: String,
    pub evidence_path: String,
    pub repo_root: String,
    pub git_branch: String,
    pub anthropic_model: String,
    pub ollama_model: String,
    pub health_port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            db_path: env::var("DB_PATH").unwrap_or_else(|_| "dual_agents.db".to_string()),
            evidence_path: env::var("EVIDENCE_PATH").unwrap_or_else(|_| "evidence".to_string()),
            repo_root: env::var("REPO_ROOT").unwrap_or_else(|_| ".".to_string()),
            git_branch: env::var("GIT_BRANCH").unwrap_or_else(|_| "main".to_string()),
            anthropic_model: env::var("ANTHROPIC_MODEL")
                .unwrap_or_else(|_| "claude-3-5-sonnet".to_string()),
            ollama_model: env::var("OLLAMA_MODEL").unwrap_or_else(|_| "codellama".to_string()),
            health_port: env::var("HEALTH_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(4001),
        }
    }
}
